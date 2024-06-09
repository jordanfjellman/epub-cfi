use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{alphanumeric1, digit0, digit1, u32, u8},
    combinator::{map, opt},
    multi::{many0, separated_list1},
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
    let (input, local_path) = many0(step)(input)?;
    Ok((input, LocalPath::new(local_path, RedirectedPath)))
}

fn path(input: &str) -> IResult<&str, Path> {
    let (input, (step, local_path)) = tuple((step, local_path))(input)?;
    Ok((input, Path::new(step, local_path)))
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
    use crate::syntax::Path;

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
    fn test_parser_local_path() {
        assert_eq!(
            local_path("/2").unwrap(),
            ("", LocalPath::new(vec![Step::new(2, None)], RedirectedPath))
        );
        assert_eq!(
            local_path("/6/4/2").unwrap(),
            (
                "",
                LocalPath::new(
                    vec![Step::new(6, None), Step::new(4, None), Step::new(2, None)],
                    RedirectedPath
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
                    LocalPath::new(vec![Step::new(2, None)], RedirectedPath)
                ))
            )
        );
        assert_eq!(
            fragment("epubcfi(/6/2[2])").unwrap(),
            (
                "",
                Fragment::new(Path::new(
                    Step::new(6, None),
                    LocalPath::new(
                        vec![Step::new(
                            2,
                            Some(Assertion::new(None, Some("2".to_string())))
                        )],
                        RedirectedPath
                    )
                ))
            )
        );
    }
}
