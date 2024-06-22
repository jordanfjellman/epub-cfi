use core::panic;

use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{alphanumeric1, digit1, u32, u8},
    combinator::{map, opt},
    multi::{many1, separated_list1},
    number::complete::float,
    sequence::{delimited, preceded, separated_pair, tuple},
    IResult,
};

use crate::syntax::*;

fn offset(input: &str) -> IResult<&str, Offset> {
    alt((temporal_offset, spatial_offset, character_offset))(input)
}

fn character_offset(input: &str) -> IResult<&str, Offset> {
    let (input, point) = preceded(tag(":"), u32)(input)?;
    let (input, assertion) = opt(assertion)(input)?;
    Ok((input, CharacterOffset::new(point, assertion).to_offset()))
}

fn spatial_offset(input: &str) -> IResult<&str, Offset> {
    let (input, (start, end)) =
        preceded(tag("@"), separated_pair(float, tag(":"), opt(float)))(input)?;
    let (input, maybe_assertion) = opt(assertion)(input)?;
    Ok((
        input,
        SpatialOffset::new(start, end, maybe_assertion).to_offset(),
    ))
}

fn temporal_offset(input: &str) -> IResult<&str, Offset> {
    let (input, offset) = preceded(tag("~"), float)(input)?;
    let (input, maybe_spatial_range) =
        opt(preceded(tag("@"), separated_pair(float, tag(":"), float)))(input)?;
    let (input, maybe_assertion) = opt(assertion)(input)?;
    Ok((
        input,
        TemporalOffset::new(offset, maybe_spatial_range, maybe_assertion).to_offset(),
    ))
}

/// A `step` starts with a slash, followed by an `integer` and an optional `assertion`.
///
/// See [Step] for more details.
pub fn step(input: &str) -> IResult<&str, Step> {
    let (input, step_size) = preceded(tag("/"), u8)(input)?;
    let (input, maybe_assertion) = opt(assertion)(input)?;
    Ok((input, Step::new(step_size, maybe_assertion)))
}

fn assertion(input: &str) -> IResult<&str, Assertion> {
    let (input, (params, value)) = delimited(tag("["), params_or_value, tag("]"))(input)?;
    Ok((
        input,
        Assertion::new(
            params.map(|p| {
                p.iter()
                    .map(|&pair| {
                        let (k, v) = pair;
                        (k.to_string(), v.to_string())
                    })
                    .collect()
            }),
            value.map(|s| s.to_string()),
        ),
    ))
}

fn parameter(input: &str) -> IResult<&str, (&str, &str)> {
    separated_pair(alphanumeric1, tag("="), alphanumeric1)(input)
}

fn parameter1(input: &str) -> IResult<&str, Vec<(&str, &str)>> {
    separated_list1(tag(";"), parameter)(input)
}

fn params_or_value(input: &str) -> IResult<&str, (Option<Vec<(&str, &str)>>, Option<&str>)> {
    alt((
        map(parameter1, |params| (Some(params), None)),
        map(digit1, |value| (None, Some(value))),
    ))(input)
}

fn local_path(input: &str) -> IResult<&str, LocalPath> {
    let (input, steps) = many1(step)(input)?;
    let (input, other) = alt((
        map(redirected_path, |p| (Some(p), None)),
        map(opt(offset), |o| (None, o)),
    ))(input)?;

    match other {
        (Some(p), None) => Ok((input, LocalPath::new_with_redirected_path(steps, p))),
        (None, o) => Ok((input, LocalPath::new_with_offset(steps, o))),
        _ => panic!("Unrecoverable state with local_path paser"), // todo: handle with nom::Err::Failure
    }
}

fn redirected_path(input: &str) -> IResult<&str, RedirectedPath> {
    let (input, (maybe_path, maybe_offset)) = preceded(tag("!"), path_or_offset)(input)?;
    Ok((
        input,
        RedirectedPath::new(Box::new(maybe_offset), Box::new(maybe_path)),
    ))
}

fn path_or_offset(input: &str) -> IResult<&str, (Option<Path>, Option<Offset>)> {
    alt((
        map(path, |p| (Some(p), None)),
        map(offset, |o| (None, Some(o))),
    ))(input)
}

fn path(input: &str) -> IResult<&str, Path> {
    let (input, (step, local)) = tuple((step, local_path))(input)?;
    Ok((input, Path::new(step, local)))
}

fn range(input: &str) -> IResult<&str, Range> {
    let (input, (start, end)) =
        preceded(tag(","), separated_pair(local_path, tag(","), local_path))(input)?;
    Ok((input, Range::new(start, end)))
}

fn fragment(input: &str) -> IResult<&str, Fragment> {
    let (input, path) = preceded(
        tag("epubcfi"),
        delimited(
            tag("("),
            // tuple(Path::from_str, opt(Range::from_str)),
            path,
            tag(")"),
        ),
    )(input)?;
    Ok((input, Fragment::new(path)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parser_character_offset() {
        assert_eq!(
            character_offset(":10").unwrap(),
            ("", CharacterOffset::new(10, None).to_offset())
        );
    }

    #[test]
    fn test_parser_spatial_offset() {
        assert_eq!(
            spatial_offset("@2.5:5.3").unwrap(),
            ("", SpatialOffset::new(2.5, Some(5.3), None).to_offset())
        )
    }

    #[test]
    fn test_parser_temporal_offset() {
        assert_eq!(
            temporal_offset("~3.7").unwrap(),
            ("", TemporalOffset::new(3.7, None, None).to_offset())
        )
    }

    #[test]
    fn test_offset() {
        assert_eq!(
            offset("~2@0.5:1.5[type=note;id=note1]").unwrap(),
            (
                "",
                Offset::Temporal(TemporalOffset::new(
                    2.0,
                    Some((0.5, 1.5)),
                    Some(Assertion::new(
                        Some(vec!(
                            ("type".to_string(), "note".to_string()),
                            ("id".to_string(), "note1".to_string())
                        )),
                        None
                    ))
                ))
            )
        );
        assert_eq!(
            offset(":10[lang=en]").unwrap(),
            (
                "",
                Offset::Character(CharacterOffset::new(
                    10,
                    Some(Assertion::new(
                        Some(vec!(("lang".to_string(), "en".to_string()))),
                        None
                    ))
                ))
            )
        );
        assert_eq!(
            offset(":1[8]").unwrap(),
            (
                "",
                Offset::Character(CharacterOffset::new(
                    1,
                    Some(Assertion::new(None, Some("8".to_string())))
                ))
            )
        );
    }

    #[test]
    fn test_parser_step() {
        assert_eq!(step("/6").unwrap(), ("", Step::new(6, None)));
        assert_eq!(
            step("/28[2]").unwrap(),
            (
                "",
                Step::new(28, Some(Assertion::new(None, Some("2".to_string()))))
            )
        );
    }

    #[test]
    fn test_parser_parameter() {
        let (input, parsed) = parameter("id=section1").unwrap();
        assert_eq!("", input);
        assert_eq!(("id", "section1"), parsed);
    }

    #[test]
    fn test_parser_parameter1() {
        let (input, parsed) = parameter1("id=section1;class=image").unwrap();
        assert_eq!("", input);
        assert_eq!(vec![("id", "section1"), ("class", "image")], parsed);
    }

    #[test]
    fn test_parser_params_or_value() {
        let (input, (maybe_params, maybe_value)) = params_or_value("8").unwrap();
        assert_eq!("", input);
        assert_eq!(None, maybe_params);
        assert_eq!(Some("8"), maybe_value);

        // numbers are placed first to confirm that they do not parse as digits
        let (input, (maybe_params, maybe_value)) =
            params_or_value("1key=1value;2key=2value").unwrap();
        assert_eq!("", input);
        assert_eq!(
            Some(vec![("1key", "1value"), ("2key", "2value")]),
            maybe_params
        );
        assert_eq!(None, maybe_value);
    }

    #[test]
    fn test_parser_assertion() {
        let result = assertion("[]");
        assert!(result.is_err());

        // most of the assertion logic is tested with the params_or_value
        // this is a sanity check
        let (input, (maybe_params, maybe_value)) = params_or_value("8").unwrap();
        assert_eq!("", input);
        assert_eq!(None, maybe_params);
        assert_eq!(Some("8"), maybe_value);
    }

    #[test]
    fn test_parser_redirected_path() {
        assert_eq!(
            redirected_path("!/4/1"),
            Ok((
                "",
                RedirectedPath::new(
                    Box::new(None),
                    Box::new(Some(Path::new(
                        Step::new(4, None),
                        LocalPath::new_with_offset(vec![Step::new(1, None)], None)
                    )))
                )
            ))
        );
        assert_eq!(
            redirected_path("!/4/1:10"),
            Ok((
                "",
                RedirectedPath::new(
                    Box::new(None),
                    Box::new(Some(Path::new(
                        Step::new(4, None),
                        LocalPath::new_with_offset(
                            vec![Step::new(1, None)],
                            Some(CharacterOffset::new(10, None).to_offset())
                        )
                    )))
                )
            ))
        );
    }

    #[test]
    fn test_parser_local_path() {
        assert_eq!(
            local_path("/2").unwrap(),
            (
                "",
                LocalPath::new_with_offset(vec![Step::new(2, None)], None)
            )
        );
        assert_eq!(
            local_path("/6/4/2").unwrap(),
            (
                "",
                LocalPath::new_with_offset(
                    vec![Step::new(6, None), Step::new(4, None), Step::new(2, None)],
                    None,
                )
            )
        );
    }

    #[test]
    fn test_parser_path() {
        assert_eq!(
            path("/6/4/2").unwrap(),
            (
                "",
                Path::new(
                    Step::new(6, None),
                    LocalPath::new_with_offset(vec![Step::new(4, None), Step::new(2, None)], None)
                )
            )
        )
    }

    #[test]
    fn test_parser_range() {
        assert_eq!(
            range(",/6/4,/6/14").unwrap(),
            (
                "",
                Range::new(
                    LocalPath::new_with_offset(vec![Step::new(6, None), Step::new(4, None)], None),
                    LocalPath::new_with_offset(vec![Step::new(6, None), Step::new(14, None)], None)
                )
            )
        );
    }

    #[test]
    fn test_parser_fragment_simple() {
        assert_eq!(
            fragment("epubcfi(/6/2)").unwrap(),
            (
                "",
                Fragment::new(Path::new(
                    Step::new(6, None),
                    LocalPath::new_with_offset(vec![Step::new(2, None)], None)
                ))
            )
        );
        assert_eq!(
            fragment("epubcfi(/6/2[2])").unwrap(),
            (
                "",
                Fragment::new(Path::new(
                    Step::new(6, None),
                    LocalPath::new_with_offset(
                        vec![Step::new(
                            2,
                            Some(Assertion::new(None, Some("2".to_string())))
                        )],
                        None
                    )
                ))
            )
        );
    }

    #[test]
    fn test_parser_fragment_complex() {
        assert_eq!(
            fragment("epubcfi(/6/2!/4/1:5)").unwrap(),
            (
                "",
                Fragment::new(Path::new(
                    Step::new(6, None),
                    LocalPath::new_with_redirected_path(
                        vec![Step::new(2, None)],
                        RedirectedPath::new(
                            Box::new(None),
                            Box::new(Some(Path::new(
                                Step::new(4, None),
                                LocalPath::new_with_offset(
                                    vec![Step::new(1, None)],
                                    Some(CharacterOffset::new(5, None).to_offset())
                                )
                            )))
                        )
                    )
                ))
            )
        );
        // assert_eq!(
        //     fragment("epubcfi(/4[lang=en]/2[role=section]/6/3!/5:10)").unwrap(),
        //     (
        //         "",
        //         Fragment::new(Path::new(
        //             Step::new(6, None),
        //             LocalPath::new(vec![Step::new(2, None)], RedirectedPath::new(None, None))
        //         ))
        //     )
        // );
        // assert_eq!(
        //     fragment("epubcfi(/2/4/8[role=note]@3.5:7.2)").unwrap(),
        //     (
        //         "",
        //         Fragment::new(Path::new(
        //             Step::new(6, None),
        //             LocalPath::new(vec![Step::new(2, None)], RedirectedPath::new(None, None))
        //         ))
        //     )
        // );
        // assert_eq!(
        //     fragment("epubcfi(/3/1!/7[lang=fr]/2~2.7)").unwrap(),
        //     (
        //         "",
        //         Fragment::new(Path::new(
        //             Step::new(6, None),
        //             LocalPath::new(vec![Step::new(2, None)], RedirectedPath::new(None, None))
        //         ))
        //     )
        // );
        // assert_eq!(
        //     fragment("epubcfi(/5/6[role=chapter]/2[lang=es]/3[role=section];id=sec2)").unwrap(),
        //     (
        //         "",
        //         Fragment::new(Path::new(
        //             Step::new(6, None),
        //             LocalPath::new(vec![Step::new(2, None)], RedirectedPath::new(None, None))
        //         ))
        //     )
        // );
    }
}
