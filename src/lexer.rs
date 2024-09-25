use std::{
    collections::VecDeque, mem::swap, vec::IntoIter
};

use crate::{
    defs::{FileMap, Location, Macro, MacroArg, Token, TokenType},
    globals::error,
};

pub struct Lexer {
    global_lexer: RegionalLexer,
    regional_lexers: VecDeque<RegionalLexer>,
    pub file_map: FileMap,
    pub macros: Vec<Macro>,
    pub ignore_stringize: bool,
}

impl Lexer {
    pub fn new(file_map: FileMap, source: &str) -> Self {
        let global_lexer = RegionalLexer::new(file_map.clone(), 0, source.to_owned());

        Self {
            global_lexer,
            regional_lexers: VecDeque::new(),
            file_map,
            macros: vec![],
            ignore_stringize: false,
        }
    }

    fn next_token(&mut self) {
        if let Some(regional) = self.regional_lexers.back_mut() {
            regional.lex_token();
        } else {
            self.global_lexer.lex_token();
        }
    }

    pub fn lex_token(&mut self) -> TokenType {
        self.next_token();

        let token_type = self.current_token_type();

        match token_type {
            TokenType::TEof => {
                if !self.regional_lexers.is_empty() {
                    // escapes current region
                    self.regional_lexers.pop_back();
                    return self.lex_token();
                }
            }
            TokenType::TIdentifier => {
                if self.expand_token() {
                    return self.lex_token();
                }
            }
            _ => {
                if self.current_regional_lexer().is_in_macro() {
                    if self.substitute_token() {
                        return self.lex_token();
                    }
                }
            }
        }

        token_type
    }

    /// Tokenizes next token. This does not attempt to expand / manipulate token with
    /// any macros, such as `#`, `##` etc.
    ///
    /// Usually used in preprocessor metalanguage parsing.
    pub fn lex_token_raw(&mut self) -> TokenType {
        self.next_token();

        let token_type = self.current_token_type();

        match token_type {
            TokenType::TEof => {
                if !self.regional_lexers.is_empty() {
                    // escapes current region
                    self.regional_lexers.pop_back();
                    return self.lex_token_raw();
                }
            }
            _ => {}
        }

        token_type
    }

    /// Attempts to expand a token, returns true if the expansion succeeded, false otherwise.
    pub fn expand_token(&mut self) -> bool {
        let ident_token = self.current_token();

        if let Some(mac) = self.find_macro(&ident_token.literal).cloned() {
            if mac.function_like && self.lex_peek_raw(TokenType::TOpenBracket) {
                return false;
            }

            let ident_token = ident_token.clone();

            if !mac.function_like {
                let file_idx = self.current_regional_lexer().file_idx;
                self.lex_expect_raw(TokenType::TIdentifier);
                self.append_regional_token_lexer(file_idx, ident_token, mac.replacement, vec![]);
                return true;
            } else {
                self.lex_expect_raw(TokenType::TIdentifier);
                self.lex_expect_raw(TokenType::TOpenBracket);
                let args = self.read_macro_args(&mac);
                self.append_regional_token_lexer(
                    self.current_file_idx(),
                    ident_token,
                    mac.replacement,
                    args,
                );
            }
        }

        false
    }

    pub fn read_macro_args(&mut self, mac: &Macro) -> Vec<MacroArg> {
        let mut args = vec![];

        for para_name in mac.parameters.iter() {
            if !args.is_empty() {
                self.lex_expect_raw(TokenType::TComma);
            }
            args.push(self.read_macro_arg(&para_name, false));
        }

        if mac.is_variadic() {
            if !self.lex_peek_raw(TokenType::TCloseBracket) && !mac.parameters.is_empty() {
                self.lex_expect_raw(TokenType::TComma);
            }

            let mut va_args_arg = self.read_macro_arg(&mac.va_args_name, true);
            va_args_arg.is_va_args = true;
            va_args_arg.omit_comma = self.lex_peek_raw(TokenType::TCloseBracket);
        }

        // Next token is potentially another macro expansion
        self.lex_expect(TokenType::TCloseBracket);

        args
    }

    pub fn read_macro_arg(&mut self, name: &str, read_rest: bool) -> MacroArg {
        let mut arg_tokens = vec![];
        let mut bracket_depth = 0;

        loop {
            if self.lex_peek_raw(TokenType::TEof) {
                error(
                    &self.file_map.borrow(),
                    &self.current_token().loc,
                    self.regional_source(),
                    "Untermintated macro argument",
                );
            }

            if bracket_depth == 0
                && (self.lex_peek(TokenType::TCloseBracket)
                    || (!read_rest && self.lex_peek(TokenType::TComma)))
            {
                break;
            }

            if self.lex_peek(TokenType::TOpenBracket) {
                bracket_depth += 1;
            }
            if self.lex_peek(TokenType::TCloseBracket) {
                bracket_depth -= 1;
            }

            arg_tokens.push(self.current_token().clone());
            self.lex_token_raw();
        }

        MacroArg::new(name.to_owned(), false, false, arg_tokens)
    }

    pub fn substitute_token(&mut self) -> bool {
        if self.lex_accept(TokenType::TCppdHash) {
            // Stringize operator `#`
            let ident_token = self.current_token().clone();
            
            if let Some(arg) = self.find_macro_arg(&ident_token.literal).cloned() {
                self.stringize(&ident_token, &arg.clone().replacement);
                
                return true;
            } else {
                error(
                    &self.file_map.borrow(),
                    &self.current_token_loc(),
                    &self.global_source(),
                    "Cannot stringize any token but a macro parameter",
                );
            }
        }

        if self.lex_peek(TokenType::TCppdHashHash) {
            let Some(prev_token) = &self.current_regional_lexer().prev_token.clone()
            else {
                error(
                    &self.file_map.borrow(),
                    &self.current_token_loc(),
                    &self.global_source(),
                    "Cannot concat tokens while `##` is at the start of macro expansion"
                );
            };

            self.lex_expect(TokenType::TCppdHashHash);

            let next_token = self.current_token().clone();

            if let Some(arg) = self.find_macro_arg(&next_token.literal).cloned() {
                let rhs = Self::join_tokens(&arg.replacement);
            }

            // FIXME: Check Rhs here in future migration
            // Reason: We don't implement boundry checking 
            // here is due to the ideology provided by rust,
            // which is quite hard to have a good way to access
            // previous and next element without any performance 
            // penalty.

        }

        false
    }

    pub fn join_tokens(tokens: &[Token]) -> String {
        let mut builder = String::with_capacity(tokens.len() * 2);

        for token in tokens {
            if !builder.is_empty() {
                builder.push(' ');
            }

            builder.push_str(&token.literal);
        }

        builder
    }

    pub fn stringize(&mut self, arg_token_loc: &Token, replacement: &[Token]) {
        let string = format!("\"{}\"", Self::join_tokens(replacement));
        let string_token = Token::new(string, TokenType::TString, arg_token_loc.loc.clone());
        
        // Consume the macro argument identifier here to prevent
        // retrieve macro arguments after escaping macro token lexer
        self.lex_expect(TokenType::TIdentifier);

        self.append_regional_token_lexer(
            self.current_file_idx(),
            arg_token_loc.clone(),
            vec![string_token],
            vec![],
        );
    }

    /// Tokenizes next token and expand into a string token if the following conditions are met:
    /// 1. Token has type TIdentifier
    /// 2. Expands to its alias once if identifier tokens
    // pub fn lex_token_then_expand_as_string(&mut self) {
    //     self.lex_token(false);

    //     let token_type = self.current_token_type();
    //     let token_str = self.current_token_literal().to_owned();
    //     let token_loc = self.current_token().loc.clone();

    //     self.expand_token(&token_str);
    //     self.lex_token(false);

    //     let region_str = format!("\"{}\"", &self.current_regional_lexer().source);
    //     self.regional_lexers.pop_back();

    //     self.current_mut_regional_lexer().cur_token =
    //         Token::new(region_str, TokenType::TString, token_loc);
    // }

    pub fn lex_accept(&mut self, token_type: TokenType) -> bool {
        if self.current_token_type() == token_type {
            self.lex_token();
            return true;
        }

        return false;
    }

    pub fn lex_accept_raw(&mut self, token_type: TokenType) -> bool {
        if self.current_token_type() == token_type {
            self.lex_token_raw();
            return true;
        }

        false
    }

    /// Peeks next token and assert if the providing token type matches next
    /// token's type.
    #[inline(always)]
    pub fn lex_peek(&self, token_type: TokenType) -> bool {
        self.current_token_type() == token_type
    }

    /// Effectively equals to [Lexer::lex_peek].
    #[inline(always)]
    pub fn lex_peek_raw(&self, token_type: TokenType) -> bool {
        self.lex_peek(token_type)
    }

    pub fn lex_expect(&mut self, token_type: TokenType) {
        if self.current_token_type() != token_type {
            error(
                &self.file_map.borrow(),
                &self.current_token().loc,
                &self.regional_source(),
                &format!(
                    "Unexpected token {:?}, expects {:?}",
                    self.current_token_type(),
                    token_type
                ),
            )
        }

        self.lex_token();
    }

    pub fn lex_expect_raw(&mut self, token_type: TokenType) {
        if self.current_token_type() != token_type {
            error(
                &self.file_map.borrow(),
                &self.current_token().loc,
                &self.regional_source(),
                &format!(
                    "Unexpected token {:?}, expects {:?}",
                    self.current_token_type(),
                    token_type
                ),
            )
        }

        self.lex_token_raw();
    }

    // ---=== Utility Functions ===--- //

    pub fn current_regional_lexer(&self) -> &RegionalLexer {
        self.regional_lexers.back().unwrap_or(&self.global_lexer)
    }

    pub fn current_mut_regional_lexer(&mut self) -> &mut RegionalLexer {
        self.regional_lexers
            .back_mut()
            .unwrap_or(&mut self.global_lexer)
    }

    pub fn current_token(&self) -> &Token {
        &self.current_regional_lexer().cur_token
    }

    pub fn current_token_type(&self) -> TokenType {
        self.current_token().token_type
    }

    pub fn current_token_literal(&self) -> &str {
        &self.current_token().literal
    }

    pub fn current_token_loc(&self) -> Location {
        self.current_regional_lexer().location()
    }

    pub fn current_file_idx(&self) -> usize {
        self.current_regional_lexer().file_idx
    }

    /// Returns current token's starting position.
    pub fn pos(&self) -> usize {
        self.current_regional_lexer().pos
    }

    pub fn global_source(&self) -> String {
        self.global_lexer.source.clone()
    }

    pub fn regional_source(&self) -> String {
        (&self.current_regional_lexer().source).into()
    }

    fn append_regional_source_lexer(&mut self, file_idx: usize, source: String) {
        self.regional_lexers
            .push_back(RegionalLexer::new(self.file_map.clone(), file_idx, source));
    }

    fn append_regional_token_lexer(
        &mut self,
        file_idx: usize,
        macro_name_token: Token,
        tokens: Vec<Token>,
        macro_args: Vec<MacroArg>,
    ) {
        self.regional_lexers
            .push_back(RegionalLexer::new_token_lexer(
                self.file_map.clone(),
                file_idx,
                macro_name_token,
                tokens,
                macro_args,
            ));
    }

    pub fn add_macro(&mut self, r#macro: Macro) {
        self.macros.push(r#macro);
    }

    pub fn find_macro(&self, name: &str) -> Option<&Macro> {
        self.macros.iter().find(|m| m.name == name)
    }

    pub fn find_macro_arg(&self, name: &str) -> Option<&MacroArg> {
        self.current_regional_lexer().macro_args.iter().find(|arg| arg.name == name)
    }
}

/// Indicates if lexer should use token iterator as backing or source to tokenize.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LexerMode {
    Source,
    Token,
}

pub struct RegionalLexer {
    files: FileMap,
    file_idx: usize,
    lexer_mode: LexerMode,
    /// Source-specialized lexer fields
    line: usize,
    source: String,
    // Refers to source code index under 
    // LexerMode::Source; refers to `tokens`
    // index under LexerMode::Token.
    pos: usize,
    cur_token: Token,
    pub skip_backslash_newline: bool,
    pub preproc_match: bool,
    // Macro-specialized lexer fields
    pub macro_name_token: Option<Token>,
    pub macro_args: Vec<MacroArg>,
    pub prev_token: Option<Token>,
    tokens: IntoIter<Token>,
}

impl RegionalLexer {
    pub fn new(files: FileMap, file_idx: usize, source: String) -> Self {
        Self {
            files,
            source,
            lexer_mode: LexerMode::Source,
            file_idx,
            line: 1,
            pos: 0,
            cur_token: Token::new("", TokenType::TStart, Location::new(file_idx, 1, 0)),
            skip_backslash_newline: true,
            preproc_match: false,
            macro_name_token: None,
            macro_args: Vec::new(),
            prev_token: None,
            tokens: Vec::new().into_iter(),
        }
    }

    pub fn new_token_lexer(
        files: FileMap,
        file_idx: usize,
        macro_name_token: Token,
        tokens: Vec<Token>,
        macro_args: Vec<MacroArg>,
    ) -> Self {
        Self {
            files,
            source: String::new(),
            lexer_mode: LexerMode::Token,
            file_idx,
            line: 0,     // Unused, use cur_token to get line number instead.
            pos: 0,      // Unused, use cur_token to get position instead.
            cur_token: Token::new("", TokenType::TStart, Location::new(file_idx, 1, 0)),
            skip_backslash_newline: true,
            preproc_match: false,
            macro_name_token: Some(macro_name_token),
            macro_args,
            prev_token: None,
            tokens: tokens.into_iter(),
        }
    }

    pub fn location(&self) -> Location {
        match self.lexer_mode {
            LexerMode::Source => self.cur_token.loc.clone(),
            LexerMode::Token => self.macro_name_token.as_ref().unwrap().loc.clone(),
        }
    }

    pub fn is_in_macro(&self) -> bool {
        self.lexer_mode == LexerMode::Token
    }

    fn is_whitespace(ch: u8) -> bool {
        ch == b' ' || ch == b'\t'
    }

    fn is_newline(ch: u8) -> bool {
        ch == b'\n' || ch == b'\r'
    }

    fn is_alnum(ch: u8) -> bool {
        ch >= b'a' && ch <= b'z'
            || ch >= b'A' && ch <= b'Z'
            || ch >= b'0' && ch <= b'9'
            || ch == b'_'
    }

    fn is_digit(ch: u8) -> bool {
        ch >= b'0' && ch <= b'9'
    }

    fn is_hex(ch: u8) -> bool {
        ch >= b'0' && ch <= b'9'
            || ch >= b'a' && ch <= b'f'
            || ch >= b'A' && ch <= b'F'
            || ch == b'x'
    }

    fn is_numeric(buf: &[u8]) -> bool {
        let mut hex = false;
        let size = buf.len();

        if size > 2 {
            hex = buf.starts_with(b"0x");
        }

        for i in 0..size {
            if hex && !Self::is_hex(buf[i]) {
                return false;
            }
            if !hex && !Self::is_digit(buf[i]) {
                return false;
            }
        }

        true
    }

    fn peek_char(&self, offset: usize) -> u8 {
        if self.pos + offset >= self.source.len() {
            return b'\0';
        }

        self.source.as_bytes()[self.pos + offset]
    }

    fn read_char(&mut self, offset: usize) {
        self.pos += offset;
    }

    fn skip_whitespaces(&mut self) {
        loop {
            let ch = self.peek_char(0);

            if self.skip_backslash_newline && Self::is_newline(ch) {
                self.line += 1;
                self.pos += 1;
                continue;
            }

            if Self::is_whitespace(ch) || self.skip_backslash_newline && ch == b'\\' {
                self.pos += 1;
                continue;
            }

            break;
        }
    }

    fn next_token(&mut self) {
        if self.lexer_mode == LexerMode::Token {
            if let Some(mut token) = self.tokens.next() {
                // Swaps out self.cur_token to variable token then replace to
                // self.prev_token if self.cur_token is not an starting token
                swap(&mut self.cur_token, &mut token);
                
                if token.token_type != TokenType::TStart {
                    self.prev_token = Some(token);
                }
            } else if self.cur_token.token_type != TokenType::TEof {
                self.prev_token = Some(self.cur_token.clone());
                self.cur_token = Token::new("", TokenType::TEof, self.cur_token.loc.clone());
            }
            return;
        }

        self.skip_whitespaces();
        let mut ch = self.peek_char(0);

        if ch == b'#' {
            if self.peek_char(1) == b'#' {
                self.make_token(TokenType::TCppdHashHash, 2);
                return;
            }
            self.make_token(TokenType::TCppdHash, 1);
            return;
        }

        if ch == b'/' {
            if self.peek_char(1) == b'*' {
                let mut enclosed = false;
                let mut offset = 2;

                while self.peek_char(offset) != b'\0' {
                    if self.peek_char(offset) == b'*' && self.peek_char(offset + 1) == b'/' {
                        enclosed = true;
                        break;
                    } else {
                        offset += 1;
                    }
                }

                if !enclosed {
                    error(
                        &self.files.borrow(),
                        &Location::new(self.file_idx, self.line, self.pos),
                        &self.source,
                        "Unenclosed comment",
                    );
                } else {
                    self.read_char(offset + 2);
                    return self.next_token();
                }
            } else {
                self.make_token(TokenType::TDivide, 1);
                return;
            }
        }

        if Self::is_digit(ch) {
            let mut length = 1;

            while Self::is_hex(self.peek_char(length)) {
                length += 1;
            }

            self.make_token(TokenType::TNumeric, length);
            return;
        }

        if ch == b'(' {
            self.make_token(TokenType::TOpenBracket, 1);
            return;
        }

        if ch == b')' {
            self.make_token(TokenType::TCloseBracket, 1);
            return;
        }

        if ch == b'{' {
            self.make_token(TokenType::TOpenCurly, 1);
            return;
        }

        if ch == b'}' {
            self.make_token(TokenType::TCloseCurly, 1);
            return;
        }

        if ch == b'[' {
            self.make_token(TokenType::TOpenSquare, 1);
            return;
        }

        if ch == b']' {
            self.make_token(TokenType::TCloseSquare, 1);
            return;
        }

        if ch == b',' {
            self.make_token(TokenType::TComma, 1);
            return;
        }

        if ch == b'^' {
            self.make_token(TokenType::TBitXor, 1);
            return;
        }

        if ch == b'~' {
            self.make_token(TokenType::TBitNot, 1);
            return;
        }

        if ch == b'"' {
            let mut length = 1;

            while self.peek_char(length) != b'"' {
                ch = self.peek_char(length);

                if ch == b'\\' {
                    length += 2;
                } else {
                    length += 1;
                }
            }

            self.make_token(TokenType::TString, length);
            self.read_char(1);
            return;
        }

        if ch == b'\'' {
            let length: usize;
            ch = self.peek_char(1);

            if ch == b'\\' {
                length = 2;
            } else {
                length = 1;
            }

            if self.peek_char(length + 1) != b'\'' {
                error(
                    &self.files.borrow(),
                    &Location::new(self.file_idx, self.line, self.pos),
                    &self.source,
                    "expected \' here to enclose char literal",
                );
            }

            self.make_token(TokenType::TChar, length);
            self.read_char(1);
            return;
        }

        if ch == b'*' {
            self.make_token(TokenType::TAsterisk, 1);
            return;
        }

        if ch == b'&' {
            if self.peek_char(1) == b'&' {
                self.make_token(TokenType::TLogAnd, 2);
                return;
            }

            if self.peek_char(1) == b'=' {
                self.make_token(TokenType::TAndeq, 2);
                return;
            }

            self.make_token(TokenType::TAmpersand, 1);
            return;
        }

        if ch == b'|' {
            if self.peek_char(1) == b'|' {
                self.make_token(TokenType::TLogOr, 2);
                return;
            }

            if self.peek_char(1) == b'|' {
                self.make_token(TokenType::TOreq, 2);
                return;
            }

            self.make_token(TokenType::TBitOr, 1);
            return;
        }

        if ch == b'<' {
            if self.peek_char(1) == b'=' {
                self.make_token(TokenType::TLe, 2);
                return;
            }

            if self.peek_char(1) == b'<' {
                self.make_token(TokenType::TLshift, 2);
                return;
            }

            self.make_token(TokenType::TLt, 1);
            return;
        }

        if ch == b'%' {
            self.make_token(TokenType::TMod, 1);
            return;
        }

        if ch == b'>' {
            if self.peek_char(1) == b'=' {
                self.make_token(TokenType::TGe, 2);
                return;
            }

            if self.peek_char(1) == b'>' {
                self.make_token(TokenType::TRshift, 2);
                return;
            }

            self.make_token(TokenType::TGt, 1);
            return;
        }

        if ch == b'!' {
            if self.peek_char(1) == b'=' {
                self.make_token(TokenType::TNoteq, 2);
                return;
            }

            self.make_token(TokenType::TLogNot, 1);
            return;
        }

        if ch == b'.' {
            if self.peek_char(1) == b'.' && self.peek_char(2) == b'.' {
                self.make_token(TokenType::TElipsis, 3);
                return;
            }

            self.make_token(TokenType::TDot, 1);
            return;
        }

        if ch == b'-' {
            if self.peek_char(1) == b'>' {
                self.make_token(TokenType::TArrow, 2);
                return;
            }

            if self.peek_char(1) == b'-' {
                self.make_token(TokenType::TDecrement, 2);
                return;
            }

            if self.peek_char(1) == b'=' {
                self.make_token(TokenType::TMinuseq, 2);
                return;
            }

            self.make_token(TokenType::TMinus, 1);
            return;
        }

        if ch == b'+' {
            if self.peek_char(1) == b'+' {
                self.make_token(TokenType::TIncrement, 2);
                return;
            }

            if self.peek_char(1) == b'=' {
                self.make_token(TokenType::TPluseq, 2);
                return;
            }

            self.make_token(TokenType::TPlus, 1);
            return;
        }

        if ch == b';' {
            self.make_token(TokenType::TSemicolon, 1);
            return;
        }

        if ch == b'?' {
            self.make_token(TokenType::TQuestion, 1);
            return;
        }

        if ch == b':' {
            self.make_token(TokenType::TColon, 1);
            return;
        }

        if ch == b'=' {
            if self.peek_char(1) == b'=' {
                self.make_token(TokenType::TEq, 2);
                return;
            }

            self.make_token(TokenType::TAssign, 1);
            return;
        }

        if Self::is_alnum(ch) {
            let mut length = 1;

            while Self::is_alnum(self.peek_char(length)) {
                length += 1;
            }

            self.make_identifier_token(length);
            return;
        }

        if ch == b'\\' {
            self.make_token(TokenType::TBackslash, 1);
            return;
        }

        if Self::is_newline(ch) {
            self.line += 1;
            self.make_token(TokenType::TNewline, 1);
            return;
        }

        if ch == b'\0' {
            self.make_token(TokenType::TEof, 0);
            return;
        }

        unreachable!()
    }

    pub fn lex_token(&mut self) {
        self.next_token();
    }

    pub fn make_token(&mut self, token_type: TokenType, length: usize) {
        let loc = Location::new(self.file_idx, self.line, self.pos);
        self.cur_token = Token::new(&self.source[self.pos..self.pos + length], token_type, loc);
        self.read_char(length);
    }

    pub fn make_identifier_token(&mut self, length: usize) {
        let literal = &self.source[self.pos..self.pos + length];
        let loc = Location::new(self.file_idx, self.line, self.pos);
        let token_type = match literal {
            "if" => TokenType::TIf,
            "while" => TokenType::TWhile,
            "for" => TokenType::TFor,
            "do" => TokenType::TDo,
            "else" => TokenType::TElse,
            "return" => TokenType::TReturn,
            "typedef" => TokenType::TTypedef,
            "enum" => TokenType::TEnum,
            "struct" => TokenType::TStruct,
            "sizeof" => TokenType::TSizeof,
            "switch" => TokenType::TSwitch,
            "case" => TokenType::TCase,
            "break" => TokenType::TBreak,
            "default" => TokenType::TDefault,
            "continue" => TokenType::TContinue,
            /* Preprocessor directives */
            "include" => TokenType::TCppdInclude,
            "define" => TokenType::TCppdDefine,
            "undef" => TokenType::TCppdUndef,
            "error" => TokenType::TCppdError,
            "elif" => TokenType::TCppdElif,
            "ifdef" => TokenType::TCppdIfdef,
            "endif" => TokenType::TCppdEndif,
            _ => TokenType::TIdentifier,
        };
        self.cur_token = Token::new(literal, token_type, loc);
        self.read_char(length);
    }
}
