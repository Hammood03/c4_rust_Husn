use std::env;
use std::mem;
use std::ptr;
use libc::{open, read, close, malloc, memset, free, printf, memcmp};

type int = i64;

static mut P: *mut u8 = ptr::null_mut();
static mut LP: *mut u8 = ptr::null_mut();
static mut DATA: *mut u8 = ptr::null_mut();
static mut E: *mut int = ptr::null_mut();
static mut LE: *mut int = ptr::null_mut();
static mut ID: *mut int = ptr::null_mut();
static mut SYM: *mut int = ptr::null_mut();
static mut TK: int = 0;
static mut IVAL: int = 0;
static mut TY: int = 0;
static mut LOC: int = 0;
static mut LINE: int = 0;
static mut SRC: int = 0;
static mut DEBUG: int = 0;

#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenKind {
    Num = 128, Fun, Sys, Glo, Loc, Id,
    Char, Else, Enum, If, Int, Return,
    Sizeof, While, Assign, Cond, Lor, Lan,
    Or, Xor, And, Eq, Ne, Lt,
    Gt, Le, Ge, Shl, Shr, Add,
    Sub, Mul, Div, Mod, Inc, Dec, Brak,
}

#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpCode {
    Lea, Imm, Jmp, Jsr, Bz, Bnz, Ent, Adj,
    Lev, Li, Lc, Si, Sc, Psh, Or, Xor,
    And, Eq, Ne, Lt, Gt, Le, Ge,
    Shl, Shr, Add, Sub, Mul, Div,
    Mod, Open, Read, Clos, Prtf, 
    Malc, Free, Mset, Mcmp, Exit,
}

#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DataType {
    Char, Int, Ptr,
}

#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IdentOffset {
    Tk, Hash, Name, Class, Type,
    Val, HClass, HType, HVal, Idsz,
}

const CHAR: int = DataType::Char as int;
const INT: int = DataType::Int as int;
const PTR: int = DataType::Ptr as int;

const OpCode_NAMES: &str = concat!(
    "LEA ,IMM ,JMP ,JSR ,BZ  ,BNZ ,ENT ,ADJ ,LEV ,LI  ,LC  ,SI  ,SC  ,PSH ,",
    "OR  ,XOR ,AND ,EQ  ,NE  ,LT  ,GT  ,LE  ,GE  ,SHL ,SHR ,ADD ,SUB ,MUL ,DIV ,MOD ,",
    "OPEN,READ,CLOS,PRTF,MALC,FREE,MSET,MCMP,EXIT,"
);

fn next() 
{
    unsafe 
    {
        let mut pp: *mut u8;
        while {
            TK = *P as int;
            TK != 0
        } {
            P = P.offset(1);
            if TK == b'\n' as int {
                if SRC != 0 {
                    let len = P.offset_from(LP) as usize;
                    let slice = std::slice::from_raw_parts(LP, len);
                    print!("{}: ", LINE);
                    if let Ok(s) = std::str::from_utf8(slice) {
                        print!("{}", s);
                    } else {
                        for &byte in slice {
                            print!("{:02x}", byte);
                        }
                    }
                    LP = P;
                    while (LE as usize) < (E as usize) {
                        LE = LE.offset(1);
                        let off = ((*LE) as usize).wrapping_mul(5);
                        let OpCode = OpCode_NAMES.get(off..off + 4).unwrap_or("");
                        print!("{:8.4}", OpCode);
                        if *LE <= (OpCode::Adj as int) {
                            LE = LE.offset(1);
                            println!(" {}", *LE);
                        } else {
                            println!();
                        }
                    }
                }
                LINE += 1;
            } else if TK == b'#' as int {
                while *P != 0 && *P != b'\n' {
                    P = P.offset(1);
                }
            } else if (TK >= b'a' as int && TK <= b'z' as int)
                   || (TK >= b'A' as int && TK <= b'Z' as int)
                   || TK == b'_' as int {
                pp = P.offset(-1);
                while (*P >= b'a' && *P <= b'z')
                   || (*P >= b'A' && *P <= b'Z')
                   || (*P >= b'0' && *P <= b'9')
                   || *P == b'_' {
                    TK = TK.wrapping_mul(147).wrapping_add(*P as int);
                    P = P.offset(1);
                }
                TK = (TK << 6) + (P.offset_from(pp) as int);
                let mut id = SYM;
                while *id.offset(IdentOffset::Tk as isize) != 0 {
                    if TK == *id.offset(IdentOffset::Hash as isize) {
                        let name_ptr = *id.offset(IdentOffset::Name as isize) as *const u8;
                        let len = P.offset_from(pp) as usize;
                        let slice1 = std::slice::from_raw_parts(name_ptr, len);
                        let slice2 = std::slice::from_raw_parts(pp, len);
                        if slice1 == slice2 {
                            TK = *id.offset(IdentOffset::Tk as isize);
                            return;
                        }
                    }
                    id = id.offset(IdentOffset::Idsz as isize);
                }
                *id.offset(IdentOffset::Name as isize) = pp as int;
                *id.offset(IdentOffset::Hash as isize) = TK;
                *id.offset(IdentOffset::Tk as isize) = TokenKind::Id as int;
                TK = TokenKind::Id as int;
                return;
            } else if TK >= b'0' as int && TK <= b'9' as int {
                IVAL = TK - b'0' as int;
                if IVAL != 0 {
                    while *P >= b'0' && *P <= b'9' {
                        IVAL = IVAL * 10 + (*P - b'0') as int;
                        P = P.offset(1);
                    }
                } else if *P == b'x' || *P == b'X' {
                    while {
                        P = P.offset(1);
                        TK = *P as int;
                        TK != 0 &&
                        ((TK >= b'0' as int && TK <= b'9' as int)
                         || (TK >= b'a' as int && TK <= b'f' as int)
                         || (TK >= b'A' as int && TK <= b'F' as int))
                    } {
                        IVAL = IVAL * 16 + ((TK & 15) + if TK >= b'A' as int { 9 } else { 0 });
                    }
                } else {
                    while *P >= b'0' && *P <= b'7' {
                        IVAL = IVAL * 8 + (*P - b'0') as int;
                        P = P.offset(1);
                    }
                }
                TK = TokenKind::Num as int;
                return;
            } else if TK == b'/' as int {
                if *P == b'/' {
                    P = P.offset(1);
                    while *P != 0 && *P != b'\n' {
                        P = P.offset(1);
                    }
                } else {
                    TK = TokenKind::Div as int;
                    return;
                }
            } else if TK == b'\'' as int || TK == b'"' as int {
                pp = DATA;
                while *P != 0 && *P != TK as u8 {
                    IVAL = *P as int;
                    P = P.offset(1);
                    if IVAL == b'\\' as int {
                        IVAL = *P as int;
                        P = P.offset(1);
                        if IVAL == b'n' as int {
                            IVAL = b'\n' as int;
                        }
                    }
                    if TK == b'"' as int {
                        *DATA = IVAL as u8;
                        DATA = DATA.offset(1);
                    }
                }
                P = P.offset(1);
                if TK == b'"' as int {
                    IVAL = pp as int;
                } else {
                    TK = TokenKind::Num as int;
                }
                return;
            } else if TK == b'=' as int {
                if *P == b'=' {
                    P = P.offset(1);
                    TK = TokenKind::Eq as int;
                } else {
                    TK = TokenKind::Assign as int;
                }
                return;
            } else if TK == b'+' as int {
                if *P == b'+' {
                    P = P.offset(1);
                    TK = TokenKind::Inc as int;
                } else {
                    TK = TokenKind::Add as int;
                }
                return;
            } else if TK == b'-' as int {
                if *P == b'-' {
                    P = P.offset(1);
                    TK = TokenKind::Dec as int;
                } else {
                    TK = TokenKind::Sub as int;
                }
                return;
            } else if TK == b'!' as int {
                if *P == b'=' {
                    P = P.offset(1);
                    TK = TokenKind::Ne as int;
                }
                return;
            } else if TK == b'<' as int {
                if *P == b'=' {
                    P = P.offset(1);
                    TK = TokenKind::Le as int;
                } else if *P == b'<' {
                    P = P.offset(1);
                    TK = TokenKind::Shl as int;
                } else {
                    TK = TokenKind::Lt as int;
                }
                return;
            } else if TK == b'>' as int {
                if *P == b'=' {
                    P = P.offset(1);
                    TK = TokenKind::Ge as int;
                } else if *P == b'>' {
                    P = P.offset(1);
                    TK = TokenKind::Shr as int;
                } else {
                    TK = TokenKind::Gt as int;
                }
                return;
            } else if TK == b'|' as int {
                if *P == b'|' {
                    P = P.offset(1);
                    TK = TokenKind::Lor as int;
                } else {
                    TK = TokenKind::Or as int;
                }
                return;
            } else if TK == b'&' as int {
                if *P == b'&' {
                    P = P.offset(1);
                    TK = TokenKind::Lan as int;
                } else {
                    TK = TokenKind::And as int;
                }
                return;
            } else if TK == b'^' as int {
                TK = TokenKind::Xor as int;
                return;
            } else if TK == b'%' as int {
                TK = TokenKind::Mod as int;
                return;
            } else if TK == b'*' as int {
                TK = TokenKind::Mul as int;
                return;
            } else if TK == b'[' as int {
                TK = TokenKind::Brak as int;
                return;
            } else if TK == b'?' as int {
                TK = TokenKind::Cond as int;
                return;
            } else if TK == b'~' as int
                   || TK == b';' as int
                   || TK == b'{' as int
                   || TK == b'}' as int
                   || TK == b'(' as int
                   || TK == b')' as int
                   || TK == b']' as int
                   || TK == b',' as int
                   || TK == b':' as int {
                return;
            }
        }
    }
}