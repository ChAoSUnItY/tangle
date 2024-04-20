use crate::{
    defs::Alias,
    globals::error,
    lexer::{Lexer, TokenType},
};

pub struct Parser {
    lexer: Lexer,
}

impl Parser {
    pub fn new(source: &str) -> Self {
        Self {
            lexer: Lexer::new(source),
        }
    }

    pub fn read_global_statements(&mut self) -> String {
        let mut builder = String::new();
        self.lexer.lex_token(true); // Skip TStart

        while self.lexer.current_token_type() != TokenType::TEof {
            if self.read_preproc_directive() {
                self.lexer.current_token_type();
                continue;
            } else if self.lexer.lex_peek(TokenType::TIdentifier) {
                // We just assume it's a macro invocation atm
                self.read_macro_invocation();
                self.lexer.lex_token(true);
                continue;
            } else {
                builder.push_str(&self.lexer.current_token_str());
                println!(
                    "{:?}: {:?}",
                    self.lexer.current_token_type(),
                    self.lexer.current_token_str()
                );
                self.lexer.lex_token(true);
            }
        }

        return builder;
    }

    #[allow(unused_variables)]
    pub fn read_macro_invocation(&mut self) {
        let alias = self.lexer.current_token_str();
        self.lexer.lex_expect(TokenType::TIdentifier, false);
        self.lexer.lex_expect(TokenType::TOpenBracket, true);
        let mut argument_regions = vec![];
        let mut replacement_builder = String::new();
        let mut bracket_depth = 0;

        while !self.lexer.lex_peek(TokenType::TEof) {
            let region = self.lexer.regional_source();
            let region_alias = self.lexer.regional_aliases();
            let token_type = self.lexer.current_token_type();
            let token_str = self.lexer.current_token_str();
            let pos = self.lexer.pos();

            if self.lexer.lex_peek(TokenType::TOpenBracket) {
                bracket_depth += 1;
            } else if self.lexer.lex_peek(TokenType::TCloseBracket) {
                bracket_depth -= 1;
            }

            if self.lexer.lex_peek(TokenType::TIdentifier) {
                self.read_macro_invocation();
            } else {
                replacement_builder.push_str(&self.lexer.current_token_str());
                self.lexer.lex_token(true);
            }

            if bracket_depth == 0 {
                if self.lexer.lex_accept(TokenType::TComma, true) {
                    argument_regions.push(replacement_builder.to_owned());
                    replacement_builder.clear();
                    continue;
                } else if self.lexer.lex_peek(TokenType::TCloseBracket) {
                    argument_regions.push(replacement_builder.to_owned());
                    break;
                }
            }
        }

        if self.lexer.lex_peek(TokenType::TEof) {
            error(
                &self.lexer.regional_source(),
                "Unexpected source end",
                self.lexer.current_token_pos(),
            );
        }

        let Some(mac) = self.lexer.find_macro(&alias) else {
            panic!("Macro {alias} is not defined but yet used");
        };

        let mut aliases = mac.parameters.clone();

        if aliases.len() != argument_regions.len() {
            panic!(
                "Macro {alias} has mismatched parameter list:\nExpects: {:?}, Got: {:?}",
                aliases.iter().map(|a| &a.alias).collect::<Vec<_>>(),
                argument_regions
            );
        }

        for (i, alias) in aliases.iter_mut().enumerate() {
            alias.replacement = argument_regions[i].to_owned();
        }

        self.lexer
            .append_regional_lexer(mac.source_span.clone(), aliases);
    }

    pub fn read_preproc_directive(&mut self) -> bool {
        if self.lexer.lex_accept(TokenType::TCppdDefine, false) {
            let alias = self.lexer.current_token_str();
            self.lexer.lex_expect(TokenType::TIdentifier, false);
            let start_pos = self.lexer.current_token_pos();
            self.lexer.current_mut_regional_lexer().skip_newline = false;

            if self.lexer.lex_accept(TokenType::TOpenBracket, false) {
                self.lexer.current_mut_regional_lexer().skip_backslash = false;

                // Macro
                let mut parameters = vec![];

                if !self.lexer.lex_accept(TokenType::TCloseBracket, false) {
                    loop {
                        let alias = self.lexer.current_token_str();
                        self.lexer.lex_expect(TokenType::TIdentifier, false);

                        // We don't care the alias region now, it is
                        // later replaced with actual parsed argument
                        parameters.push(Alias::new(alias, String::new()));

                        if self.lexer.lex_accept(TokenType::TComma, false) {
                            continue;
                        } else {
                            self.lexer.lex_expect(TokenType::TCloseBracket, false);
                            break;
                        }
                    }
                }

                let start_pos = self.lexer.current_token_pos();

                while !self.lexer.lex_peek(TokenType::TNewline) {
                    if self.lexer.lex_accept(TokenType::TBackslash, true) {
                        self.lexer.lex_expect(TokenType::TNewline, true);
                    } else {
                        self.lexer.lex_token(false);
                    }
                }

                let end_pos = self.lexer.current_token_pos();
                self.lexer.current_mut_regional_lexer().skip_newline = true;
                self.lexer.lex_expect(TokenType::TNewline, false);

                self.lexer.add_macro(
                    &alias,
                    parameters,
                    self.lexer.global_source()[start_pos..end_pos].to_string(),
                );
            } else {
                while !self.lexer.lex_peek(TokenType::TNewline) {
                    self.lexer.lex_token(false);
                }

                let end_pos = self.lexer.current_token_pos();
                self.lexer.current_mut_regional_lexer().skip_newline = true;
                self.lexer.lex_expect(TokenType::TNewline, false);

                self.lexer.add_alias(
                    &alias,
                    self.lexer.global_source()[start_pos..end_pos].to_string(),
                );
            }

            return true;
        }

        return false;
    }
}
