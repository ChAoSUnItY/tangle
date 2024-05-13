use crate::{
    defs::Alias,
    globals::error,
    lexer::{Lexer, TokenType},
    source::SourceSegments,
};

pub struct Parser<'src> {
    lexer: Lexer<'src>,
}

impl<'src> Parser<'src> {
    pub fn new(source: &'src str) -> Self {
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
                builder.push_str(&Into::<String>::into(&self.lexer.current_token_str()));
                println!(
                    "{:?}: {}",
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
        let mut replacement_builder = SourceSegments::new(&[]);
        let mut argument_regions = vec![];
        let mut bracket_depth = 0;

        while !self.lexer.lex_peek(TokenType::TEof) {
            // let region = self.lexer.regional_source();
            // let region_alias = self.lexer.regional_aliases();
            // let token_type = self.lexer.current_token_type();
            // let token_str = self.lexer.current_token_str();
            // let pos = self.lexer.pos();

            if self.lexer.lex_peek(TokenType::TOpenBracket) {
                bracket_depth += 1;
            } else if self.lexer.lex_peek(TokenType::TCloseBracket) {
                bracket_depth -= 1;
            }

            if self.lexer.lex_peek(TokenType::TIdentifier) {
                self.read_macro_invocation();
            } else {
                replacement_builder.push_segment(&self.lexer.current_token_str());
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
                &Into::<String>::into(self.lexer.regional_source()),
                "Unexpected source end",
                self.lexer.current_token_pos(),
            );
        }

        let Some(mac) = self.lexer.find_macro(alias.clone()) else {
            panic!("Macro {alias} is not defined but yet used");
        };

        let mut aliases = mac.parameters.clone();

        if !mac.is_variadic && aliases.len() != argument_regions.len() {
            panic!(
                "Macro {alias} has mismatched parameter list:\nExpects: {:?}, Got: {:?}",
                aliases.iter().map(|a| &a.alias).collect::<Vec<_>>(),
                argument_regions
            );
        }

        if mac.is_variadic && argument_regions.len() < aliases.len() - 1 {
            panic!(
                "Macro {alias} takes at least {:?} arguments",
                aliases.len() - 1
            );
        }

        if mac.is_variadic {
            if argument_regions.len() == aliases.len() - 1 {
                // Appends synthesized parameter
                argument_regions.push(SourceSegments::new(&[]));
            } else {
                // Concats remaining parameters into single parameter
                let mut variadic_argument = argument_regions[aliases.len() - 1].to_owned();

                for i in aliases.len()..argument_regions.len() {
                    variadic_argument.push_span(",");
                    variadic_argument.push_segment(&argument_regions[i]);
                }

                argument_regions.truncate(aliases.len());
                argument_regions[aliases.len() - 1] = variadic_argument;
            }
        }

        for (i, alias) in aliases.iter_mut().enumerate() {
            alias.replacement = argument_regions[i].clone();
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
                let mut is_variadic = false;
                self.lexer.current_mut_regional_lexer().skip_backslash = false;

                // Macro
                let mut parameters = vec![];

                if !self.lexer.lex_accept(TokenType::TCloseBracket, false) {
                    loop {
                        let alias = if self.lexer.lex_accept(TokenType::TElipsis, false) {
                            is_variadic = true;
                            SourceSegments::new(&[b"__VA_ARGS__"])
                        } else {
                            let alias = self.lexer.current_token_str();
                            self.lexer.lex_expect(TokenType::TIdentifier, false);
                            alias
                        };

                        // We don't care the alias region now, it is
                        // later replaced with actual parsed argument
                        parameters.push(Alias::new(alias, SourceSegments::new(&[])));

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

                // Validate if __VA_ARGS__ is at the end of parameter list
                if is_variadic {
                    let va_args_parameters = parameters
                        .iter()
                        .enumerate()
                        .filter(|(_, parameter)| parameter.alias == "__VA_ARGS__")
                        .collect::<Vec<_>>();

                    if va_args_parameters.len() != 1 {
                        error(
                            &Into::<String>::into(self.lexer.regional_source()),
                            "__VA_ARGS__ cannot be declared more than once",
                            start_pos,
                        );
                    }

                    let (param_idx, _) = va_args_parameters.first().unwrap();

                    if *param_idx != parameters.len() - 1 {
                        error(
                            &Into::<String>::into(self.lexer.regional_source()),
                            "__VA_ARGS__ must be defined at the end of macro parameter list",
                            start_pos,
                        );
                    }
                }

                self.lexer.add_macro(
                    alias,
                    parameters,
                    is_variadic,
                    self.lexer.global_source().index_range(start_pos..end_pos),
                );
            } else {
                while !self.lexer.lex_peek(TokenType::TNewline) {
                    self.lexer.lex_token(false);
                }

                let end_pos = self.lexer.current_token_pos();
                self.lexer.current_mut_regional_lexer().skip_newline = true;
                self.lexer.lex_expect(TokenType::TNewline, false);

                self.lexer.add_alias(
                    alias,
                    self.lexer.global_source().index_range(start_pos..end_pos),
                );
            }

            return true;
        }

        return false;
    }
}
