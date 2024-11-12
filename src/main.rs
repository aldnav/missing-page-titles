use nom::bytes::complete::tag;
use nom::bytes::complete::take_until;
use nom::sequence::delimited;
use nom::IResult;

/// Try to parse title from the same line as the title block
fn parse_title_block_same_line(input: &str) -> IResult<&str, &str> {
    let start_parser = take_until("{% block title %}")(input)?;
    let (starting_point, _) = start_parser;
    if starting_point.is_empty() {
        return Ok((input, ""));
    }
    let (remainder, try_title_from_same_line) = delimited(
        tag("{% block title %}"),
        take_until("{% endblock %}"),
        tag("{% endblock %}"),
    )(starting_point)?;
    match try_title_from_same_line {
        "" => Ok((input, "")),
        _ => Ok((remainder, try_title_from_same_line.trim())),
    }
}

fn has_title(input: &str) -> bool {
    match parse_title_block_same_line(input) {
        Ok((_, title)) => !title.is_empty(),
        Err(_) => false,
    }
}

fn main() {
    has_title(&std::env::args().nth(1).expect("No input provided"));
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error;

    #[test]
    fn parse_title_block_same_line_valid() -> Result<(), Box<dyn Error>> {
        let input = r#"
        {% extends "base.html" %}
        {% block title %} {% trans "¡Bienvenidos a mi perfil!" %} - {{ block.super }} {% endblock %}
        {% block content %} This is the content {% endblock %}
        "#;
        let result = parse_title_block_same_line(input);
        assert!(result.is_ok());
        let (remaining, title) = result?;
        assert!(remaining.contains("{% block content %} This is the content {% endblock %}"));
        assert_eq!(
            title,
            r#"{% trans "¡Bienvenidos a mi perfil!" %} - {{ block.super }}"#
        );
        assert!(has_title(input));
        Ok(())
    }

    #[test]
    fn parse_title_block_same_line_valid_but_empty() -> Result<(), Box<dyn Error>> {
        let input = r#"
        {% block title %}{% endblock %}
        "#;
        let result = parse_title_block_same_line(input);
        assert!(result.is_ok());
        let (remaining, title) = result?;
        assert!(remaining.contains("{% block title %}{% endblock %}"));
        assert_eq!(title, "");
        assert_eq!(has_title(input), false);
        Ok(())
    }

    #[test]
    fn test_parse_title_multiline_block_valid() -> Result<(), Box<dyn Error>> {
        let input = r#"{% extends "base.html" %}{% block title %}
        My tweets
        {% endblock %}{% block content %}Hi {{ person|default:"friend" }}!{% endblock %}
        "#;
        let result = parse_title_block_same_line(input);
        assert!(result.is_ok());
        let (remaining, title) = result?;
        assert!(remaining
            .contains("{% block content %}Hi {{ person|default:\"friend\" }}!{% endblock %}"));
        assert_eq!(title, "My tweets");
        assert!(has_title(input));
        Ok(())
    }

    // #[test]
    // fn test_parse_title_block_partial() {
    //     let input = "Title: ";
    //     let result = parse_title_block(input);
    //     assert!(result.is_err());
    //     let error = result.unwrap_err();
    //     assert_eq!(error.code, ErrorKind::Eof);
    // }
}
