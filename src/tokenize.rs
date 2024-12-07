use crate::tokenize::TokenizeError::UnfinishedLiteralValue;
use std::char::ParseCharError;
use std::num::ParseFloatError;
use std::str::Chars;

/// Takes in an input string and returns a Vector of Token
pub fn tokenize(input: String) -> Result<Vec<Token>, TokenizeError> {
    let chars: Vec<char> = input.chars().collect();
    let mut index = 0;

    let mut tokens = Vec::new();
    while index < chars.len() {
        let token = make_token(&chars, &mut index)?;
        tokens.push(token);
        index += 1;
    }

    Ok(tokens)
}

fn make_token(chars: &[char], index: &mut usize) -> Result<Token, TokenizeError> {
    let mut c = chars[*index];

    while c.is_ascii_whitespace() {
        *index += 1;
        if *index >= chars.len() {
            return Err(TokenizeError::UnexpectedEof);
        }
        c = chars[*index];
    }
    let token = match c {
        '[' => Token::LeftBracket,
        ']' => Token::RightBracket,
        '{' => Token::LeftBrace,
        '}' => Token::RightBrace,
        ',' => Token::Comma,
        ':' => Token::Colon,

        'n' => tokenize_literal(chars, index, String::from("null"), Token::Null)?,
        't' => tokenize_literal(chars, index, String::from("true"), Token::True)?,
        'f' => tokenize_literal(chars, index, String::from("false"), Token::False)?,

        c if c.is_ascii_digit() => tokenize_float(chars, index)?,

        '"' => tokenize_string(chars, index)?,
        c => return Err(TokenizeError::CharNotRecognized(c)),
    };

    Ok(token)
}

fn tokenize_string(chars: &[char], current_index: &mut usize) -> Result<Token, TokenizeError> {
    // New string buffer
    let mut string = String::new();
    let mut is_escaping = false;

    // Loop through from the current index to the end of the chars length
    loop {
        *current_index += 1;
        // if we get to the end of the buffer and there is no closing "
        // it is deemed invalid json and we throw an error
        if *current_index >= chars.len() {
            return Err(TokenizeError::UnclosedQuotes);
        }
        let ch = chars[*current_index];
        match ch {
            // if it is the end of a string and we are not escaping break
            '"' if !is_escaping => break,
            // toggle the escaping based on the forward slash
            '\\' => is_escaping = !is_escaping,
            // else stop escaping
            _ => is_escaping = false,
        }
        string.push(ch);
    }

    // return the string token
    Ok(Token::String(string))
}

fn tokenize_float(chars: &[char], curr_index: &mut usize) -> Result<Token, TokenizeError> {
    // string to stored an unparsed number
    let mut unparsed_num = String::new();
    // flag to set if its a float or not
    let mut has_decimal = false;
    // flag if negative number
    let mut has_negative = false;

    // walks through the characters starting at the index
    while *curr_index < chars.len() {
        let ch = chars[*curr_index];

        match ch {
            // if the character is a digit we add it to our string
            c if c.is_ascii_digit() => unparsed_num.push(c),
            // if its a decimal we set the has_decimal flag and then add the decimal to the string
            c if c == '.' && !has_decimal => {
                unparsed_num.push('.');
                has_decimal = true;
            }
            c if c == '-' && !has_decimal => {
                unparsed_num.push('-');
                has_negative = true;
            }
            // if we reach the end of the number we terminate, say a bracket or whitespace
            _ => break,
        }
        *curr_index += 1;
    }

    match unparsed_num.parse() {
        Ok(f) => Ok(Token::Number(f)),
        Err(err) => Err(TokenizeError::ParseNumberError(err)),
    }
}

fn tokenize_literal(
    chars: &[char],
    index: &mut usize,
    string_value: String,
    token: Token,
) -> Result<Token, TokenizeError> {
    for expected_char in string_value.chars() {
        if expected_char != chars[*index] {
            return Err(TokenizeError::UnfinishedLiteralValue);
        }
        *index += 1;
    }
    // when you get a successful case, you have to go back one character so that you don't skip future single characters
    *index -= 1;
    Ok(token)
}

/// Possible errors from attempting to parse JSON
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum TokenizeError {
    UnfinishedLiteralValue,
    ParseNumberError(ParseFloatError),
    UnclosedQuotes,
    UnexpectedEof,
    CharNotRecognized(char),
}

#[derive(Debug, PartialEq)]
pub enum Token {
    /// `{`
    LeftBrace,
    /// `}`
    RightBrace,
    /// `[`
    LeftBracket,
    /// `]`
    RightBracket,
    /// `,`
    Comma,
    /// `:`
    Colon,
    /// `null`
    Null,
    /// `false`
    False,
    /// `true`
    True,
    /// Any number literal
    Number(f64),
    /// Key of the key/value pair or string value
    String(String),
}
#[cfg(test)]
impl Token {
    fn string(input: &str) -> Self {
        Self::String(String::from(input))
    }
}

#[cfg(test)]
mod tests {
    use super::{tokenize, Token, TokenizeError};

    #[test]
    fn true_comma() {
        let input = String::from("true,");
        let expected = [Token::True, Token::Comma];

        let actual = tokenize(input).unwrap();

        assert_eq!(actual, expected);
    }

    #[test]
    fn just_comma() {
        let input = String::from(",");
        let expected = [Token::Comma];

        let actual = tokenize(input).unwrap();

        assert_eq!(actual, expected);
    }

    #[test]
    fn just_null() {
        let input = String::from("null");
        let expected = [Token::Null];

        let actual = tokenize(input).unwrap();

        assert_eq!(actual, expected);
    }

    #[test]
    fn just_true() {
        let input = String::from("true");
        let expected = [Token::True];

        let actual = tokenize(input).unwrap();

        assert_eq!(actual, expected);
    }

    #[test]
    fn just_false() {
        let input = String::from("false");
        let expected = [Token::False];

        let actual = tokenize(input).unwrap();

        assert_eq!(actual, expected);
    }

    #[test]
    fn all_punctuation() {
        let input = String::from("[]{},:");

        let expected = [
            Token::LeftBracket,
            Token::RightBracket,
            Token::LeftBrace,
            Token::RightBrace,
            Token::Comma,
            Token::Colon,
        ];

        let actual = tokenize(input).unwrap();

        assert_eq!(actual, expected);
    }

    #[test]
    fn integer() {
        let input = String::from("100");

        let expected = [Token::Number(100.0)];

        let actual = tokenize(input).unwrap();

        assert_eq!(actual, expected)
    }

    #[test]
    fn float() {
        let input = String::from("1.23");

        let expected = [Token::Number(1.23)];

        let actual = tokenize(input).unwrap();

        assert_eq!(actual, expected)
    }

    #[test]
    fn simple_string() {
        let input = String::from("\"ken\"");
        let expected = [Token::string("ken")];

        let actual = tokenize(input).unwrap();

        assert_eq!(actual, expected);
    }

    #[test]
    fn unterminated_string() {
        let input = String::from("\"ken");
        let expected = Err(TokenizeError::UnclosedQuotes);

        let actual = tokenize(input);

        assert_eq!(actual, expected);
    }

    #[test]
    fn escaped_quote() {
        let input = String::from(r#""the \" is OK""#);
        let expected = [Token::String(String::from(r#"the \" is OK"#))];

        let actual = tokenize(input).unwrap();

        assert_eq!(actual, expected);
    }

    #[test]
    fn white_space() {
        let input = String::from(" ");
        let expected = Err(TokenizeError::UnexpectedEof);

        let actual = tokenize(input);

        assert_eq!(actual, expected);
    }

    #[test]
    fn more_complex() {
        let input = String::from("{\"key\": \"value\"}");
        let expected = [
            Token::LeftBrace,
            Token::string("key"),
            Token::Colon,
            Token::string("value"),
            Token::RightBrace,
        ];

        let actual = tokenize(input).unwrap();

        assert_eq!(actual, expected);
    }
}
