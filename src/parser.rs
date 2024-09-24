use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{
    defs::{FileMap, Macro, TokenType},
    globals::error,
    lexer::Lexer,
};

pub struct Parser {
    file_map: FileMap,
    lexer: Lexer,
}

impl Parser {
    pub fn new(file_name: &str, source: &str) -> Self {
        let file_map = Rc::new(RefCell::new(vec![file_name.to_owned()]));
        let lexer = Lexer::new(file_map.clone(), source);

        Self { file_map, lexer }
    }

    pub fn read_global_statements(&mut self) -> String {
        let mut builder = String::new();
        self.lexer.lex_token(); // Skip TStart

        while self.lexer.current_token_type() != TokenType::TEof {
            if self.read_preproc_directive() {
                println!("=======>");
                println!("{:#?}", self.lexer.macros);
                continue;
            }

            println!("{:?}", self.lexer.current_token());
            builder.push_str(self.lexer.current_token_literal());
            builder.push('\n');
            self.lexer.lex_token();
        }

        return builder;
    }

    // #[allow(unused_variables)]
    // pub fn read_macro_invocation(&mut self, mac: Macro) {
    //     self.lexer.lex_expect(TokenType::TIdentifier, false);
    //     self.lexer.lex_expect(TokenType::TOpenBracket, true);
    //     let mut argument_regions = vec![];
    //     let mut replacement_builder = String::new();
    //     let mut bracket_depth = 0;

    //     while !self.lexer.lex_peek(TokenType::TEof) {
    //         if self.lexer.lex_peek(TokenType::TOpenBracket) {
    //             bracket_depth += 1;
    //         } else if self.lexer.lex_peek(TokenType::TCloseBracket) {
    //             bracket_depth -= 1;
    //         }

    //         replacement_builder.push_str(&self.lexer.current_token_literal());
    //         self.lexer.lex_token(true);

    //         if bracket_depth == 0 {
    //             if self.lexer.lex_accept(TokenType::TComma, true) {
    //                 argument_regions.push(replacement_builder.to_owned());
    //                 replacement_builder.clear();
    //                 continue;
    //             } else if self.lexer.lex_peek(TokenType::TCloseBracket) {
    //                 argument_regions.push(replacement_builder.to_owned());
    //                 break;
    //             }
    //         }
    //     }

    //     let mut parameters = mac.parameters.clone();

    //     if !mac.is_variadic() && parameters.len() != argument_regions.len() {
    //         panic!(
    //             "Macro {} has mismatched parameter list:\nExpects exact {} arguments but got {} arguments",
    //             mac.name,
    //             parameters.len(),
    //             argument_regions.len()
    //         );
    //     }

    //     if mac.is_variadic() && argument_regions.len() < parameters.len() - 1 {
    //         panic!(
    //             "Macro {} takes at least {:?} arguments",
    //             mac.name,
    //             parameters.len() - 1
    //         );
    //     }

    //     if mac.is_variadic() {
    //         if argument_regions.len() == parameters.len() - 1 {
    //             // Appends synthesized parameter
    //             argument_regions.push(String::new());
    //         } else {
    //             // Concats remaining parameters into single parameter
    //             let variadic_argument = argument_regions[parameters.len() - 1..].join(",");
    //             argument_regions.truncate(parameters.len());
    //             argument_regions[parameters.len() - 1] = variadic_argument;
    //         }
    //     }

    //     let alias_map = parameters.into_iter().zip(argument_regions).collect::<HashMap<_, _>>();

    //     self.lexer
    //         .append_regional_lexer(self.lexer.current_regional_lexer().file_idx, mac.replacement.clone());
    // }

    pub fn read_preproc_directive(&mut self) -> bool {
        if self.lexer.lex_accept_raw(TokenType::TCppdHash) {
            if self.lexer.lex_accept_raw(TokenType::TCppdDefine) {
                let name_token = self.lexer.current_token().clone();
                let name = name_token.literal.clone();
                self.lexer.lex_expect_raw(TokenType::TIdentifier);
                self.lexer
                    .current_mut_regional_lexer()
                    .skip_backslash_newline = false;
                self.lexer.ignore_stringize = true;

                if self.lexer.lex_accept_raw(TokenType::TOpenBracket) {
                    let mut is_variadic = false;

                    // Macro
                    let mut parameters = vec![];

                    if !self.lexer.lex_accept_raw(TokenType::TCloseBracket) {
                        loop {
                            if self.lexer.lex_peek_raw(TokenType::TElipsis) {
                                is_variadic = true;
                                let elipsis_token = self.lexer.current_token().clone();
                                self.lexer.lex_expect_raw(TokenType::TElipsis);
                                self.lexer.lex_expect_raw(TokenType::TCloseBracket);
                                parameters.push(elipsis_token);
                                break;
                            }

                            let parameter_token = self.lexer.current_token().clone();
                            self.lexer.lex_expect_raw(TokenType::TIdentifier);

                            parameters.push(parameter_token);

                            if self.lexer.lex_accept_raw(TokenType::TComma) {
                                continue;
                            } else {
                                self.lexer.lex_expect_raw(TokenType::TCloseBracket);
                                break;
                            }
                        }
                    }

                    let mut body_tokens = vec![];

                    while !self.lexer.lex_peek_raw(TokenType::TNewline) {
                        if self.lexer.lex_accept_raw(TokenType::TBackslash) {
                            self.lexer.lex_expect_raw(TokenType::TNewline);
                        } else {
                            body_tokens.push(self.lexer.current_token().clone());
                            self.lexer.lex_token_raw();
                        }
                    }

                    self.lexer
                        .current_mut_regional_lexer()
                        .skip_backslash_newline = true;
                    self.lexer.ignore_stringize = false;

                    // Validate if __VA_ARGS__ is at the end of parameter list
                    if is_variadic {
                        let va_args_parameters = parameters
                            .iter()
                            .enumerate()
                            .filter(|(_, token)| token.token_type == TokenType::TElipsis)
                            .collect::<Vec<_>>();

                        let (param_idx, token) = va_args_parameters.first().unwrap();

                        if *param_idx != parameters.len() - 1 {
                            error(
                                &self.file_map.borrow(),
                                &token.loc,
                                &self.lexer.regional_source(),
                                "__VA_ARGS__ must be defined at the end of macro parameter list",
                            );
                        }
                    }

                    // Add alias first then resolve next potential macro
                    self.lexer.add_macro(Macro::new_macro(
                        name,
                        parameters.into_iter().map(|token| token.literal).collect(),
                        if is_variadic { String::from("__VA_ARGS__") } else { String::new() },
                        body_tokens,
                    ));

                    // Next token may be replaced with macro
                    self.lexer.lex_expect(TokenType::TNewline);
                } else {
                    let mut body_tokens = vec![];

                    while !self.lexer.lex_peek_raw(TokenType::TNewline) {
                        if self.lexer.lex_accept_raw(TokenType::TBackslash) {
                            self.lexer.lex_expect_raw(TokenType::TNewline);
                        } else {
                            body_tokens.push(self.lexer.current_token().clone());
                            self.lexer.lex_token_raw();
                        }
                    }

                    self.lexer
                        .current_mut_regional_lexer()
                        .skip_backslash_newline = true;

                    // Add alias first then resolve next potential macro
                    self.lexer.add_macro(Macro::new_alias(
                        name,
                        body_tokens,
                    ));

                    // Next token may be replaced with macro
                    self.lexer.lex_expect(TokenType::TNewline);
                }

                return true;
            }
        }

        false
    }
}
