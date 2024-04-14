use crate::lexer::{TokenType, Lexer};

pub struct Parser<'src> {
    lexer: Lexer<'src>,
}

impl<'src> Parser<'src> {
    pub fn new(source: &'src str) -> Self {
        Self {
            lexer: Lexer::new(source)
        }
    }

    pub fn read_global_statements(&mut self) {
        self.lexer.lex_token(true);

        while self.lexer.current_token_type() != TokenType::TEof {
            if self.read_preproc_directive() {
                self.lexer.current_token_type();
            } else {
                let token = self.lexer.lex_token(true);
                println!("{:?}", token);
            }
        }
    }

    pub fn read_preproc_directive(&mut self) -> bool {
        if self.lexer.lex_accept(TokenType::TCppdDefine) {
            let alias = self.lexer.current_token_str();
            self.lexer.lex_expect(TokenType::TIdentifier, false);
            let start_pos = self.lexer.current_token_pos();
            self.lexer.current_mut_regional_lexer().skip_newline = false;
            
            while !self.lexer.lex_peek(TokenType::TNewline) {
                self.lexer.lex_token(false);
            }

            let end_pos = self.lexer.current_token_pos();
            self.lexer.current_mut_regional_lexer().skip_newline = true;
            self.lexer.lex_expect(TokenType::TNewline, false);

            self.lexer.add_alias(alias, &self.lexer.global_source()[start_pos..end_pos]);
            return true;
        }

        return false;
    }
}
