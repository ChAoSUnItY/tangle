use std::{cell::RefCell, collections::HashMap, rc::Rc};

pub type FileMap = Rc<RefCell<Vec<String>>>;

#[derive(Debug, Clone)]
pub struct Location {
    pub file_idx: usize,
    pub line: usize,
    /// Refers to the relative position to the corresponding regional source
    pub pos: usize,
}

impl Location {
    pub fn new(file_idx: usize, line: usize, pos: usize) -> Self {
        Self { file_idx, line, pos }
    }
}

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
    TIf,   /* including C pre-processor directive variant */
    TElse, /* including C pre-processor directive variant */
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
    TCppdElif,
    TCppdEndif,
    TCppdIfdef,
    TCppdHash,     /* #, possibly a pre-processor directive start or stringizing operator */
    TCppdHashHash, /* ## */
    /* hints */
    TBackslash,
    TNewline,
}

#[derive(Debug, Clone)]
pub struct Token {
    pub literal: String,
    pub token_type: TokenType,
    pub loc: Location,
}

impl Token {
    pub fn new(literal: impl ToString, token_type: TokenType, loc: Location) -> Self {
        Self {
            literal: literal.to_string(),
            token_type,
            loc,
        }
    }

    pub fn new_eof(token: &Token) -> Self {
        Self {
            literal: String::new(),
            token_type: TokenType::TEof,
            loc: token.loc.clone()
        }
    }
}

pub type MacroParam = String;

#[derive(Debug, Clone)]
pub struct Macro {
    pub name: String,
    pub parameters: Vec<MacroParam>,
    pub function_like: bool,
    pub va_args_name: String,
    pub replacement: Vec<Token>,
}

impl Macro {
    pub fn new_alias(name: String, replacement: Vec<Token>) -> Self {
        Self {
            name,
            parameters: vec![],
            function_like: false,
            va_args_name: String::new(),
            replacement,
        }
    }

    pub fn new_macro(
        name: String,
        parameters: Vec<String>,
        va_args_name: String,
        replacement: Vec<Token>,
    ) -> Self {
        Self {
            name,
            parameters,
            function_like: true,
            va_args_name,
            replacement,
        }
    }

    #[inline(always)]
    pub fn is_variadic(&self) -> bool {
        self.function_like && !self.va_args_name.is_empty()
    }
}

#[derive(Debug, Clone)]
pub struct MacroArg {
    pub name: String,
    pub is_va_args: bool,
    pub omit_comma: bool,
    pub replacement: Vec<Token>,
}

impl MacroArg {
    pub fn new(name: String, is_va_args: bool, omit_comma: bool, replacement: Vec<Token>) -> Self {
        Self {
            name,
            is_va_args,
            replacement,
            omit_comma
        }
    }
}
