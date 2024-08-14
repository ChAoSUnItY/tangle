use std::collections::VecDeque;

use crate::{
    defs::{Alias, Macro},
    globals::error,
};

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenType {
    TStart, /* FIXME: it was intended to start the state machine. */
    TNumeric,
    TIdentifier,
    TComma,  /* , */
    TString, /* null-terminated string */
    TChar,
    TOpenBracket,  /* ( */
    TCloseBracket, /* ) */
    TOpenCurly,    /* { */
    TCloseCurly,   /* } */
    TOpenSquare,   /* [ */
    TCloseSquare,  /* ] */
    TAsterisk,     /* '*' */
    TDivide,       /* / */
    TMod,          /* % */
    TBitOr,        /* | */
    TBitXor,       /* ^ */
    TBitNot,       /* ~ */
    TLogAnd,       /* && */
    TLogOr,        /* || */
    TLogNot,       /* ! */
    TLt,           /* < */
    TGt,           /* > */
    TLe,           /* <= */
    TGe,           /* >= */
    TLshift,       /* << */
    TRshift,       /* >> */
    TDot,          /* . */
    TArrow,        /* -> */
    TPlus,         /* + */
    TMinus,        /* - */
    TMinuseq,      /* -= */
    TPluseq,       /* += */
    TOreq,         /* |= */
    TAndeq,        /* &= */
    TEq,           /* == */
    TNoteq,        /* != */
    TAssign,       /* = */
    TIncrement,    /* ++ */
    TDecrement,    /* -- */
    TQuestion,     /* ? */
    TColon,        /* : */
    TSemicolon,    /* ; */
    TEof,          /* end-of-file (EOF) */
    TAmpersand,    /* & */
    TReturn,
    TIf,
    TElse,
    TWhile,
    TFor,
    TDo,
    TTypedef,
    TEnum,
    TStruct,
    TSizeof,
    TElipsis, /* ... */
    TSwitch,
    TCase,
    TBreak,
    TDefault,
    TContinue,
    /* C pre-processor directives */
    TCppdInclude,
    TCppdDefine,
    TCppdUndef,
    TCppdError,
    TCppdIf,
    TCppdElif,
    TCppdElse,
    TCppdEndif,
    TCppdIfdef,
    TCppdStringify, /* # */
    /* hints */
    TBackslash,
    TNewline,
}

pub struct Lexer {
    regional_lexers: VecDeque<RegionalLexer>,
    aliases: Vec<Alias>,
    macros: Vec<Macro>,
    pub ignore_stringify: bool,
}

impl Lexer {
    pub fn new(source: &str) -> Self {
        Self {
            regional_lexers: VecDeque::from(vec![RegionalLexer::new(source.to_owned(), vec![])]),
            aliases: vec![],
            macros: vec![],
            ignore_stringify: false,
        }
    }

    fn next_token(&mut self) {
        self.regional_lexers.back_mut().unwrap().lex_token();
    }

    pub fn lex_token(&mut self, aliasing: bool) -> TokenType {
        self.next_token();

        let token_type = self.current_token_type();

        match token_type {
            TokenType::TEof => {
                if self.regional_lexers.len() > 1 {
                    // esacapes current region
                    self.regional_lexers.pop_back();
                    return self.lex_token(aliasing);
                }
            }
            TokenType::TIdentifier => {
                if aliasing {
                    if let Some(alias) = self.find_alias(&self.current_token_str()) {
                        // enter alias region for parsing
                        self.append_regional_lexer(alias.replacement.clone(), vec![]);
                        return self.lex_token(aliasing);
                    }
                }
            }
            TokenType::TCppdStringify => {
                if self.ignore_stringify {
                    return TokenType::TCppdStringify;
                }

                self.lex_token_then_expand_as_string();
                return TokenType::TString;
            }
            _ => {}
        }

        token_type
    }

    /// Tokenizes next token and expand into a string token if the following conditions are met:
    /// 1. Token has type TIdentifier
    /// 2. Expands to its alias once if identifier tokens
    pub fn lex_token_then_expand_as_string(&mut self) {
        self.next_token();

        let token_type = self.current_token_type();

        match token_type {
            TokenType::TIdentifier => {
                if let Some(alias) = self.find_alias(&self.current_token_str()) {
                    self.current_mut_regional_lexer().cur_token_str = alias.replacement.clone();
                    self.current_mut_regional_lexer().cur_token_type = TokenType::TString;
                }
            }
            _ => {}
        }

        self.current_mut_regional_lexer().cur_token_str = format!("\"{}\"", self.current_mut_regional_lexer().cur_token_str);
    }

    pub fn lex_accept_internal(&mut self, token_type: TokenType, aliasing: bool) -> bool {
        if self.current_token_type() == token_type {
            self.lex_token(aliasing);
            return true;
        }

        return false;
    }

    pub fn lex_accept(&mut self, token_type: TokenType, aliasing: bool) -> bool {
        self.lex_accept_internal(token_type, aliasing)
    }

    pub fn lex_peek(&mut self, token_type: TokenType) -> bool {
        self.current_token_type() == token_type
    }

    pub fn lex_expect(&mut self, token_type: TokenType, aliasing: bool) {
        if self.current_token_type() != token_type {
            error(
                &self.regional_source(),
                &format!(
                    "Unexpected token {:?}, expexts {:?}",
                    self.current_token_type(),
                    token_type
                ),
                self.pos(),
            )
        }

        self.lex_token(aliasing);
    }

    pub fn current_regional_lexer(&self) -> &RegionalLexer {
        self.regional_lexers.back().unwrap()
    }

    pub fn current_mut_regional_lexer(&mut self) -> &mut RegionalLexer {
        self.regional_lexers.back_mut().unwrap()
    }

    pub fn current_token_type(&self) -> TokenType {
        self.current_regional_lexer().cur_token_type
    }

    pub fn current_token_str(&self) -> String {
        self.current_regional_lexer().cur_token_str.clone()
    }

    pub fn current_token_pos(&self) -> usize {
        self.current_regional_lexer().cur_token_pos
    }

    pub fn pos(&self) -> usize {
        self.current_regional_lexer().pos
    }

    pub fn global_source(&self) -> String {
        (&self.regional_lexers.front().unwrap().source).into()
    }

    pub fn regional_source(&self) -> String {
        (&self.current_regional_lexer().source).into()
    }

    pub fn regional_aliases(&self) -> &[Alias] {
        &self.current_regional_lexer().regional_aliases
    }

    pub fn append_regional_lexer(&mut self, source: String, regional_aliases: Vec<Alias>) {
        self.regional_lexers
            .push_back(RegionalLexer::new(source, regional_aliases));
    }

    pub fn add_alias(&mut self, alias: &str, source_span: String) {
        self.aliases
            .push(Alias::new(alias.to_string(), source_span));
    }

    pub fn find_alias(&self, alias: &str) -> Option<&Alias> {
        let resolution = self
            .aliases
            .iter()
            .find(|a| a.alias == alias && !a.disabled);

        if resolution.is_some() {
            return resolution;
        }

        return self
            .regional_lexers
            .back()
            .unwrap()
            .regional_aliases
            .iter()
            .find(|a| a.alias == alias);
    }

    pub fn undef_alias(&mut self, alias: &str) -> bool {
        for alias_instance in self.aliases.iter_mut() {
            if alias_instance.alias == alias {
                alias_instance.disabled = true;
                return true;
            }
        }

        return false;
    }

    pub fn add_macro(
        &mut self,
        name: &str,
        parameters: Vec<Alias>,
        is_variadic: bool,
        source_span: String,
    ) {
        self.macros.push(Macro::new(
            name.to_string(),
            parameters,
            is_variadic,
            source_span,
        ));
    }

    pub fn find_macro(&self, name: &str) -> Option<&Macro> {
        self.macros.iter().find(|m| m.name == name)
    }
}

pub struct RegionalLexer {
    source: String,
    pos: usize,
    regional_aliases: Vec<Alias>,
    cur_token_type: TokenType,
    cur_token_str: String,
    cur_token_pos: usize,
    pub ignore_stringify: bool,
    pub skip_newline: bool,
    pub skip_backslash: bool,
    pub preproc_match: bool,
}

impl RegionalLexer {
    pub fn new(source: String, regional_aliases: Vec<Alias>) -> Self {
        Self {
            source,
            pos: 0,
            regional_aliases,
            cur_token_type: TokenType::TStart,
            cur_token_str: String::new(),
            cur_token_pos: 0,
            ignore_stringify: false,
            skip_newline: true,
            skip_backslash: true,
            preproc_match: false,
        }
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

            if Self::is_whitespace(ch)
                || self.skip_newline && Self::is_newline(ch)
                || self.skip_backslash && ch == b'\\'
            {
                self.pos += 1;
                continue;
            }

            break;
        }
    }

    fn next_token(&mut self) -> TokenType {
        self.skip_whitespaces();
        self.cur_token_pos = self.pos;
        let start_pos = self.cur_token_pos;
        let mut ch = self.peek_char(0);

        if ch == b'#' {
            let mut length = 1;

            while Self::is_alnum(self.peek_char(length)) {
                length += 1;
            }

            self.cur_token_str = self.source[self.pos..self.pos + length].to_string();
            self.read_char(length);

            return match self.cur_token_str.as_str() {
                "#include" => TokenType::TCppdInclude,
                "#define" => TokenType::TCppdDefine,
                "#undef" => TokenType::TCppdUndef,
                "#error" => TokenType::TCppdError,
                "#if" => TokenType::TCppdIf,
                "#elif" => TokenType::TCppdElif,
                "#ifdef" => TokenType::TCppdIfdef,
                "#else" => TokenType::TCppdElse,
                "#endif" => TokenType::TCppdEndif,
                "#" => TokenType::TCppdStringify,
                _ => error(
                    &Into::<String>::into(&self.source),
                    &format!("Unexpected preprocessor directive {}", self.cur_token_str),
                    start_pos,
                ),
            };
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
                        &Into::<String>::into(&self.source),
                        "Unenclosed comment",
                        self.pos,
                    );
                } else {
                    self.read_char(offset + 2);
                    return self.next_token();
                }
            } else {
                self.read_char(1);
                return TokenType::TDivide;
            }
        }

        if Self::is_digit(ch) {
            let mut length = 1;

            while Self::is_hex(self.peek_char(length)) {
                length += 1;
            }

            self.cur_token_str = self.source[self.pos..self.pos + length].to_string();
            self.read_char(length);

            return TokenType::TNumeric;
        }

        if ch == b'(' {
            self.read_char(1);
            self.cur_token_str = "(".to_string();
            return TokenType::TOpenBracket;
        }

        if ch == b')' {
            self.read_char(1);
            self.cur_token_str = ")".to_string();
            return TokenType::TCloseBracket;
        }

        if ch == b'{' {
            self.read_char(1);
            return TokenType::TOpenCurly;
        }

        if ch == b'}' {
            self.read_char(1);
            return TokenType::TCloseCurly;
        }

        if ch == b'[' {
            self.read_char(1);
            return TokenType::TOpenSquare;
        }

        if ch == b']' {
            self.read_char(1);
            return TokenType::TCloseSquare;
        }

        if ch == b',' {
            self.read_char(1);
            self.cur_token_str = ",".to_string();
            return TokenType::TComma;
        }

        if ch == b'^' {
            self.read_char(1);
            return TokenType::TBitXor;
        }

        if ch == b'~' {
            self.read_char(1);
            return TokenType::TBitNot;
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

            self.cur_token_str = self.source[self.pos..self.pos + length].to_string();
            self.read_char(length + 1);

            return TokenType::TString;
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
                    &Into::<String>::into(&self.source),
                    "expected \' here to enclose char literal",
                    self.pos + length,
                );
            }

            self.cur_token_str = self.source[self.pos..self.pos + length].to_string();
            self.read_char(length + 1);
            return TokenType::TChar;
        }

        if ch == b'*' {
            self.read_char(1);
            return TokenType::TAsterisk;
        }

        if ch == b'&' {
            if self.peek_char(1) == b'&' {
                self.read_char(2);
                return TokenType::TLogAnd;
            }

            if self.peek_char(1) == b'=' {
                self.read_char(2);
                return TokenType::TAndeq;
            }

            self.read_char(1);
            return TokenType::TAmpersand;
        }

        if ch == b'|' {
            if self.peek_char(1) == b'|' {
                self.read_char(2);
                return TokenType::TLogOr;
            }

            if self.peek_char(1) == b'|' {
                self.read_char(2);
                return TokenType::TOreq;
            }

            self.read_char(1);
            return TokenType::TBitOr;
        }

        if ch == b'<' {
            if self.peek_char(1) == b'=' {
                self.read_char(2);
                return TokenType::TLe;
            }

            if self.peek_char(1) == b'<' {
                self.read_char(2);
                return TokenType::TLshift;
            }

            self.read_char(1);
            return TokenType::TLt;
        }

        if ch == b'%' {
            self.read_char(1);
            return TokenType::TMod;
        }

        if ch == b'>' {
            if self.peek_char(1) == b'=' {
                self.read_char(2);
                return TokenType::TGe;
            }

            if self.peek_char(1) == b'>' {
                self.read_char(2);
                return TokenType::TRshift;
            }

            self.read_char(1);
            return TokenType::TGt;
        }

        if ch == b'!' {
            if self.peek_char(1) == b'=' {
                self.read_char(2);
                return TokenType::TNoteq;
            }

            self.read_char(1);
            return TokenType::TLogNot;
        }

        if ch == b'.' {
            if self.peek_char(1) == b'.' && self.peek_char(2) == b'.' {
                self.read_char(3);
                return TokenType::TElipsis;
            }

            self.read_char(1);
            return TokenType::TDot;
        }

        if ch == b'-' {
            if self.peek_char(1) == b'>' {
                self.read_char(2);
                return TokenType::TArrow;
            }

            if self.peek_char(1) == b'-' {
                self.read_char(2);
                return TokenType::TDecrement;
            }

            if self.peek_char(1) == b'=' {
                self.read_char(2);
                return TokenType::TMinuseq;
            }

            self.read_char(1);
            return TokenType::TMinus;
        }

        if ch == b'+' {
            if self.peek_char(1) == b'+' {
                self.read_char(2);
                return TokenType::TIncrement;
            }

            if self.peek_char(1) == b'=' {
                self.read_char(2);
                return TokenType::TPluseq;
            }

            self.read_char(1);
            self.cur_token_str = "+".to_string();
            return TokenType::TPlus;
        }

        if ch == b';' {
            self.read_char(1);
            self.cur_token_str = ";".to_string();
            return TokenType::TSemicolon;
        }

        if ch == b'?' {
            self.read_char(1);
            return TokenType::TQuestion;
        }

        if ch == b':' {
            self.read_char(1);
            return TokenType::TColon;
        }

        if ch == b'=' {
            if self.peek_char(1) == b'=' {
                self.read_char(2);
                return TokenType::TEq;
            }

            self.read_char(1);
            return TokenType::TAssign;
        }

        if Self::is_alnum(ch) {
            let mut length = 1;

            while Self::is_alnum(self.peek_char(length)) {
                length += 1;
            }

            self.cur_token_str = self.source[self.pos..self.pos + length].to_string();
            self.read_char(length);

            return match self.cur_token_str.as_str() {
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
                _ => TokenType::TIdentifier,
            };
        }

        if ch == b'\\' {
            self.read_char(1);
            return TokenType::TBackslash;
        }

        if Self::is_newline(ch) {
            self.read_char(1);
            return TokenType::TNewline;
        }

        if ch == b'\0' {
            return TokenType::TEof;
        }

        unreachable!()
    }

    pub fn lex_token(&mut self) {
        self.cur_token_type = self.next_token();
    }
}
