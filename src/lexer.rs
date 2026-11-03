use logos::Logos;

#[derive(Logos, Debug, PartialEq, Eq)]
#[logos(skip(r"[ \t\r\n\x0B\x0C]+"))]
pub(crate) enum Token {
    #[regex(r"([0-9]+(\.[0-9]*)?|\.[0-9]+)([eE][+-]?[0-9]+)?")]
    Number,

    #[regex(r"[A-Za-z_][A-Za-z0-9_]*")]
    Name,

    #[token("+")]
    Plus,

    #[token("-")]
    Minus,

    #[token("*")]
    Star,

    #[token("/")]
    Slash,

    #[token("^")]
    Caret,

    #[token("(")]
    LeftParen,

    #[token(")")]
    RightParen,

    #[token(",")]
    Comma,
}

#[cfg(test)]
mod tests {
    use super::Token;
    use logos::Logos;

    #[test]
    fn preserves_command_text_and_reports_errors() {
        let mut lexer = Token::lexer("factor(16/4)+π");

        for (expected, text, span) in [
            (Token::Name, "factor", 0..6),
            (Token::LeftParen, "(", 6..7),
            (Token::Number, "16", 7..9),
            (Token::Slash, "/", 9..10),
            (Token::Number, "4", 10..11),
            (Token::RightParen, ")", 11..12),
            (Token::Plus, "+", 12..13),
        ] {
            assert_eq!(lexer.next(), Some(Ok(expected)));
            assert_eq!(lexer.slice(), text);
            assert_eq!(lexer.span(), span);
        }

        assert_eq!(lexer.next(), Some(Err(())));
        assert_eq!(lexer.slice(), "π");
        assert_eq!(lexer.span(), 13..15);
        assert_eq!(lexer.next(), None);
    }

    #[test]
    fn recognizes_number_spellings() {
        for input in [
            "0",
            "123456789012345678901234567890",
            "0.5",
            ".5",
            "1.",
            "6.02e23",
            "2E-3",
        ] {
            let mut lexer = Token::lexer(input);

            assert_eq!(lexer.next(), Some(Ok(Token::Number)), "{input}");
            assert_eq!(lexer.slice(), input);
            assert_eq!(lexer.next(), None);
        }
    }
}