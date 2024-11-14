use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::bytes::complete::take_until;
use nom::sequence::delimited;
use nom::IResult;

/// Try to parse title from the same line as the title block
fn parse_title_block_template_tag(input: &str) -> IResult<&str, &str> {
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
    if remainder == input {
        return Err(nom::Err::Error(nom::error::Error::new(
            input,
            nom::error::ErrorKind::Tag,
        )));
    }
    match try_title_from_same_line {
        "" => Ok((input, "")),
        _ => Ok((remainder, try_title_from_same_line.trim())),
    }
}

/// Try to parse title from the title HTML tag
fn parse_title_block_html_tag(input: &str) -> IResult<&str, &str> {
    let start_parser = take_until("<head>")(input)?;
    let (starting_point, _) = start_parser;
    if starting_point.is_empty() {
        return Ok((input, ""));
    }
    let (remainder, head_content) =
        delimited(tag("<head>"), take_until("</head>"), tag("</head>"))(starting_point)?;
    let title_parser = take_until("<title>")(head_content)?;
    let (title_starting_point, _) = title_parser;
    if title_starting_point.is_empty() {
        return Ok((input, ""));
    }
    let (title_remainder, try_title_from_html_tag) =
        delimited(tag("<title>"), take_until("</title>"), tag("</title>"))(title_starting_point)?;
    if title_remainder == head_content {
        return Err(nom::Err::Error(nom::error::Error::new(
            input,
            nom::error::ErrorKind::Tag,
        )));
    }
    match try_title_from_html_tag {
        "" => Ok((input, "")),
        _ => Ok((remainder, try_title_from_html_tag.trim())),
    }
}

fn has_title(input: &str) -> bool {
    let result = alt((parse_title_block_html_tag, parse_title_block_template_tag))(input);
    match result {
        Ok((_, title)) => !title.is_empty(),
        Err(_) => false,
    }
}

fn main() {
    let res = has_title(&std::env::args().nth(1).expect("No input provided"));
    std::process::exit(if res { 0 } else { 1 })
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
        let result = parse_title_block_template_tag(input);
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
        let result = parse_title_block_template_tag(input);
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
        let result = parse_title_block_template_tag(input);
        assert!(result.is_ok());
        let (remaining, title) = result?;
        assert!(remaining
            .contains("{% block content %}Hi {{ person|default:\"friend\" }}!{% endblock %}"));
        assert_eq!(title, "My tweets");
        assert!(has_title(input));
        Ok(())
    }

    #[test]
    fn test_parse_title_html_tag() -> Result<(), Box<dyn Error>> {
        let input = r#"<html>
        <title>Not the title</title>
        <head>
            <title>My videos</title>
        </head>
        <body>
            <h1>My videos</h1>
            <title>Misplaced title</title>
        </body>
        </html>"#;
        let result = parse_title_block_html_tag(input);
        assert!(result.is_ok());
        let (remaining, title) = result?;
        assert!(remaining.contains("<title>Misplaced title</title>"));
        assert_eq!(title, "My videos");
        assert!(has_title(input));
        Ok(())
    }

    #[test]
    fn test_parse_no_valid_title() -> Result<(), Box<dyn Error>> {
        let input = r#"
        {% extends "base.html" %}
        {% block content %} This is the content {% endblock %}
        "#;
        let result = parse_title_block_template_tag(input);
        assert!(result.is_ok() == false);
        assert!(has_title(input) == false);

        let input_html = r#"<html>
        <head>
            <meta charset="UTF-8">
        </head>
        <body>
            <h1>No title here</h1>
        </body>
        </html>"#;
        let result_html = parse_title_block_html_tag(input_html);
        assert!(result_html.is_ok() == false);
        assert!(has_title(input_html) == false);
        Ok(())
    }
}
