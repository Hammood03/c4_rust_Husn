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

fn expr(lev: int) 
{
    unsafe {
        // t is a temporary integer; d points into the symbol table.
        let mut t: int;
        let mut d: *mut int;

        if TK == 0 {
            println!("{}: unexpected eof in expression", LINE);
            std::process::exit(-1);
        } else if TK == TokenKind::Num as int {
            E = E.offset(1);
            *E = OpCode::Imm as int;
            E = E.offset(1);
            *E = IVAL;
            next();
            TY = INT;
        } else if TK == b'"' as int {
            E = E.offset(1);
            *E = OpCode::Imm as int;
            E = E.offset(1);
            *E = IVAL;
            next();
            while TK == b'"' as int {
                next();
            }
            // Align DATA up to a multiple of sizeof(int):
            DATA = (((DATA as usize) + mem::size_of::<int>()) & !(mem::size_of::<int>() - 1)) as *mut u8;
            TY = PTR;
        } else if TK == TokenKind::Sizeof as int {
            next();
            if TK == b'(' as int {
                next();
            } else {
                println!("{}: open paren expected in sizeof", LINE);
                std::process::exit(-1);
            }
            TY = INT;
            if TK == TokenKind::Int as int {
                next();
            } else if TK == TokenKind::Char as int {
                next();
                TY = CHAR;
            }
            while TK == TokenKind::Mul as int {
                next();
                TY = TY + PTR;
            }
            if TK == b')' as int {
                next();
            } else {
                println!("{}: close paren expected in sizeof", LINE);
                std::process::exit(-1);
            }
            E = E.offset(1);
            *E = OpCode::Imm as int;
            E = E.offset(1);
            if TY == CHAR {
                *E = mem::size_of::<u8>() as int;
            } else {
                *E = mem::size_of::<int>() as int;
            }
            TY = INT;
        } else if TK == TokenKind::Id as int {
            // Copy current id pointer in d from the global symbol table pointer.
            d = ID;
            next();
            if TK == b'(' as int {
                next();
                t = 0;
                while TK != b')' as int {
                    expr(TokenKind::Assign as int);
                    E = E.offset(1);
                    *E = OpCode::Psh as int;
                    t += 1;
                    if TK == b',' as int {
                        next();
                    }
                }
                next();
                if *d.offset(IdentOffset::Class as isize) == TokenKind::Sys as int {
                    E = E.offset(1);
                    *E = *d.offset(IdentOffset::Val as isize);
                } else if *d.offset(IdentOffset::Class as isize) == TokenKind::Fun as int {
                    E = E.offset(1);
                    *E = OpCode::Jsr as int;
                    E = E.offset(1);
                    *E = *d.offset(IdentOffset::Val as isize);
                } else {
                    println!("{}: bad function call", LINE);
                    std::process::exit(-1);
                }
                if t != 0 {
                    E = E.offset(1);
                    *E = OpCode::Adj as int;
                    E = E.offset(1);
                    *E = t;
                }
                TY = *d.offset(IdentOffset::Type as isize);
            } else if *d.offset(IdentOffset::Class as isize) == TokenKind::Num as int {
                E = E.offset(1);
                *E = OpCode::Imm as int;
                E = E.offset(1);
                *E = *d.offset(IdentOffset::Val as isize);
                TY = INT;
            } else {
                if *d.offset(IdentOffset::Class as isize) == TokenKind::Loc as int {
                    E = E.offset(1);
                    *E = OpCode::Lea as int;
                    E = E.offset(1);
                    *E = LOC - *d.offset(IdentOffset::Val as isize);
                } else if *d.offset(IdentOffset::Class as isize) == TokenKind::Glo as int {
                    E = E.offset(1);
                    *E = OpCode::Imm as int;
                    E = E.offset(1);
                    *E = *d.offset(IdentOffset::Val as isize);
                } else {
                    println!("{}: undefined variable", LINE);
                    std::process::exit(-1);
                }
                E = E.offset(1);
                TY = *d.offset(IdentOffset::Type as isize);
                if TY == CHAR {
                    *E = OpCode::Lc as int;
                } else {
                    *E = OpCode::Li as int;
                }
            }
        } else if TK == b'(' as int {
            next();
            if TK == TokenKind::Int as int || TK == TokenKind::Char as int {
                t = if TK == TokenKind::Int as int { INT } else { CHAR };
                next();
                while TK == TokenKind::Mul as int {
                    next();
                    t = t + PTR;
                }
                if TK == b')' as int {
                    next();
                } else {
                    println!("{}: bad cast", LINE);
                    std::process::exit(-1);
                }
                expr(TokenKind::Inc as int);
                TY = t;
            } else {
                expr(TokenKind::Assign as int);
                if TK == b')' as int {
                    next();
                } else {
                    println!("{}: close paren expected", LINE);
                    std::process::exit(-1);
                }
            }
        } else if TK == TokenKind::Mul as int {
            next();
            expr(TokenKind::Inc as int);
            if TY > INT {
                TY = TY - PTR;
            } else {
                println!("{}: bad dereference", LINE);
                std::process::exit(-1);
            }
            E = E.offset(1);
            if TY == CHAR {
                *E = OpCode::Lc as int;
            } else {
                *E = OpCode::Li as int;
            }
        } else if TK == TokenKind::And as int {
            next();
            expr(TokenKind::Inc as int);
            if *E == OpCode::Lc as int || *E == OpCode::Li as int {
                E = E.offset(-1);
            } else {
                println!("{}: bad address-of", LINE);
                std::process::exit(-1);
            }
            TY = TY + PTR;
        } else if TK == b'!' as int {
            next();
            expr(TokenKind::Inc as int);
            E = E.offset(1);
            *E = OpCode::Psh as int;
            E = E.offset(1);
            *E = OpCode::Imm as int;
            E = E.offset(1);
            *E = 0;
            E = E.offset(1);
            *E = TokenKind::Eq as int;
            TY = INT;
        } else if TK == b'~' as int {
            next();
            expr(TokenKind::Inc as int);
            E = E.offset(1);
            *E = OpCode::Psh as int;
            E = E.offset(1);
            *E = OpCode::Imm as int;
            E = E.offset(1);
            *E = -1;
            E = E.offset(1);
            *E = TokenKind::Xor as int;
            TY = INT;
        } else if TK == TokenKind::Add as int {
            next();
            expr(TokenKind::Inc as int);
            TY = INT;
        } else if TK == TokenKind::Sub as int {
            next();
            E = E.offset(1);
            *E = OpCode::Imm as int;
            if TK == TokenKind::Num as int {
                E = E.offset(1);
                *E = -IVAL;
                next();
            } else {
                E = E.offset(1);
                *E = -1;
                E = E.offset(1);
                *E = OpCode::Psh as int;
                expr(TokenKind::Inc as int);
                E = E.offset(1);
                *E = OpCode::Mul as int;
            }
            TY = INT;
        } else if TK == TokenKind::Inc as int || TK == TokenKind::Dec as int {
            t = TK;
            next();
            expr(TokenKind::Inc as int);
            if *E == OpCode::Lc as int {
                *E = OpCode::Psh as int;
                E = E.offset(1);
                *E = OpCode::Lc as int;
            } else if *E == OpCode::Li as int {
                *E = OpCode::Psh as int;
                E = E.offset(1);
                *E = OpCode::Li as int;
            } else {
                println!("{}: bad lvalue in pre-increment", LINE);
                std::process::exit(-1);
            }
            E = E.offset(1);
            *E = OpCode::Psh as int;
            E = E.offset(1);
            *E = OpCode::Imm as int;
            E = E.offset(1);
            if TY > PTR {
                *E = mem::size_of::<int>() as int;
            } else {
                *E = mem::size_of::<u8>() as int;
            }
            E = E.offset(1);
            if t == TokenKind::Inc as int {
                *E = TokenKind::Add as int;
            } else {
                *E = TokenKind::Sub as int;
            }
            E = E.offset(1);
            if TY == CHAR {
                *E = OpCode::Sc as int;
            } else {
                *E = OpCode::Si as int;
            }
        } else {
            println!("{}: bad expression", LINE);
            std::process::exit(-1);
        }

        // Now do the “precedence climbing” loop
        while TK >= lev {
            t = TY;
            if TK == TokenKind::Assign as int {
                next();
                if *E == OpCode::Lc as int || *E == OpCode::Li as int {
                    *E = OpCode::Psh as int;
                } else {
                    println!("{}: bad lvalue in assignment", LINE);
                    std::process::exit(-1);
                }
                expr(TokenKind::Assign as int);
                TY = t;
                E = E.offset(1);
                if TY == CHAR {
                    *E = OpCode::Sc as int;
                } else {
                    *E = OpCode::Si as int;
                }
            } else if TK == TokenKind::Cond as int {
                next();
                E = E.offset(1);
                *E = OpCode::Bz as int;
                E = E.offset(1);
                d = E;
                expr(TokenKind::Assign as int);
                if TK == b':' as int {
                    next();
                } else {
                    println!("{}: conditional missing colon", LINE);
                    std::process::exit(-1);
                }
                // Store the jump address (e+3) in *d:
                *d = (E.offset(3) as usize) as int;
                E = E.offset(1);
                *E = OpCode::Jmp as int;
                E = E.offset(1);
                d = E;
                expr(TokenKind::Cond as int);
                *d = (E.offset(1) as usize) as int;
            } else if TK == TokenKind::Lor as int {
                next();
                E = E.offset(1);
                *E = OpCode::Bnz as int;
                E = E.offset(1);
                d = E;
                expr(TokenKind::Lan as int);
                *d = (E.offset(1) as usize) as int;
                TY = INT;
            } else if TK == TokenKind::Lan as int {
                next();
                E = E.offset(1);
                *E = OpCode::Bz as int;
                E = E.offset(1);
                d = E;
                expr(TokenKind::Or as int);
                *d = (E.offset(1) as usize) as int;
                TY = INT;
            } else if TK == TokenKind::Or as int {
                next();
                E = E.offset(1);
                *E = OpCode::Psh as int;
                expr(TokenKind::Xor as int);
                E = E.offset(1);
                *E = TokenKind::Or as int;
                TY = INT;
            } else if TK == TokenKind::Xor as int {
                next();
                E = E.offset(1);
                *E = OpCode::Psh as int;
                expr(TokenKind::And as int);
                E = E.offset(1);
                *E = TokenKind::Xor as int;
                TY = INT;
            } else if TK == TokenKind::And as int {
                next();
                E = E.offset(1);
                *E = OpCode::Psh as int;
                expr(TokenKind::Eq as int);
                E = E.offset(1);
                *E = TokenKind::And as int;
                TY = INT;
            } else if TK == TokenKind::Eq as int {
                next();
                E = E.offset(1);
                *E = OpCode::Psh as int;
                expr(TokenKind::Lt as int);
                E = E.offset(1);
                *E = TokenKind::Eq as int;
                TY = INT;
            } else if TK == TokenKind::Ne as int {
                next();
                E = E.offset(1);
                *E = OpCode::Psh as int;
                expr(TokenKind::Lt as int);
                E = E.offset(1);
                *E = TokenKind::Ne as int;
                TY = INT;
            } else if TK == TokenKind::Lt as int {
                next();
                E = E.offset(1);
                *E = OpCode::Psh as int;
                expr(TokenKind::Shl as int);
                E = E.offset(1);
                *E = TokenKind::Lt as int;
                TY = INT;
            } else if TK == TokenKind::Gt as int {
                next();
                E = E.offset(1);
                *E = OpCode::Psh as int;
                expr(TokenKind::Shl as int);
                E = E.offset(1);
                *E = TokenKind::Gt as int;
                TY = INT;
            } else if TK == TokenKind::Le as int {
                next();
                E = E.offset(1);
                *E = OpCode::Psh as int;
                expr(TokenKind::Shl as int);
                E = E.offset(1);
                *E = TokenKind::Le as int;
                TY = INT;
            } else if TK == TokenKind::Ge as int {
                next();
                E = E.offset(1);
                *E = OpCode::Psh as int;
                expr(TokenKind::Shl as int);
                E = E.offset(1);
                *E = TokenKind::Ge as int;
                TY = INT;
            } else if TK == TokenKind::Shl as int {
                next();
                E = E.offset(1);
                *E = OpCode::Psh as int;
                expr(TokenKind::Add as int);
                E = E.offset(1);
                *E = TokenKind::Shl as int;
                TY = INT;
            } else if TK == TokenKind::Shr as int {
                next();
                E = E.offset(1);
                *E = OpCode::Psh as int;
                expr(TokenKind::Add as int);
                E = E.offset(1);
                *E = TokenKind::Shr as int;
                TY = INT;
            } else if TK == TokenKind::Add as int {
                next();
                E = E.offset(1);
                *E = OpCode::Psh as int;
                expr(TokenKind::Mul as int);
                TY = t;
                if TY > PTR {
                    E = E.offset(1);
                    *E = OpCode::Psh as int;
                    E = E.offset(1);
                    *E = OpCode::Imm as int;
                    E = E.offset(1);
                    *E = mem::size_of::<int>() as int;
                    E = E.offset(1);
                    *E = TokenKind::Mul as int;
                }
                E = E.offset(1);
                *E = TokenKind::Add as int;
            } else if TK == TokenKind::Sub as int {
                next();
                E = E.offset(1);
                *E = OpCode::Psh as int;
                expr(TokenKind::Mul as int);
                TY = t;
                if t > PTR && t == TY {
                    E = E.offset(1);
                    *E = TokenKind::Sub as int;
                    E = E.offset(1);
                    *E = OpCode::Psh as int;
                    E = E.offset(1);
                    *E = OpCode::Imm as int;
                    E = E.offset(1);
                    *E = mem::size_of::<int>() as int;
                    E = E.offset(1);
                    *E = TokenKind::Div as int;
                    TY = INT;
                } else if TY > PTR {
                    E = E.offset(1);
                    *E = OpCode::Psh as int;
                    E = E.offset(1);
                    *E = OpCode::Imm as int;
                    E = E.offset(1);
                    *E = mem::size_of::<int>() as int;
                    E = E.offset(1);
                    *E = TokenKind::Mul as int;
                    E = E.offset(1);
                    *E = TokenKind::Sub as int;
                } else {
                    E = E.offset(1);
                    *E = TokenKind::Sub as int;
                }
            } else if TK == TokenKind::Mul as int {
                next();
                E = E.offset(1);
                *E = OpCode::Psh as int;
                expr(TokenKind::Inc as int);
                E = E.offset(1);
                *E = TokenKind::Mul as int;
                TY = INT;
            } else if TK == TokenKind::Div as int {
                next();
                E = E.offset(1);
                *E = OpCode::Psh as int;
                expr(TokenKind::Inc as int);
                E = E.offset(1);
                *E = TokenKind::Div as int;
                TY = INT;
            } else if TK == TokenKind::Mod as int {
                next();
                E = E.offset(1);
                *E = OpCode::Psh as int;
                expr(TokenKind::Inc as int);
                E = E.offset(1);
                *E = TokenKind::Mod as int;
                TY = INT;
            } else if TK == TokenKind::Inc as int || TK == TokenKind::Dec as int {
                if *E == OpCode::Lc as int {
                    *E = OpCode::Psh as int;
                    E = E.offset(1);
                    *E = OpCode::Lc as int;
                } else if *E == OpCode::Li as int {
                    *E = OpCode::Psh as int;
                    E = E.offset(1);
                    *E = OpCode::Li as int;
                } else {
                    println!("{}: bad lvalue in post-increment", LINE);
                    std::process::exit(-1);
                }
                E = E.offset(1);
                *E = OpCode::Psh as int;
                E = E.offset(1);
                *E = OpCode::Imm as int;
                E = E.offset(1);
                if TY > PTR {
                    *E = mem::size_of::<int>() as int;
                } else {
                    *E = mem::size_of::<u8>() as int;
                }
                E = E.offset(1);
                if TK == TokenKind::Inc as int {
                    *E = TokenKind::Sub as int;
                } else {
                    *E = TokenKind::Add as int;
                }
                E = E.offset(1);
                if TY == CHAR {
                    *E = OpCode::Sc as int;
                } else {
                    *E = OpCode::Si as int;
                }
                E = E.offset(1);
                *E = OpCode::Psh as int;
                E = E.offset(1);
                *E = OpCode::Imm as int;
                E = E.offset(1);
                if TY > PTR {
                    *E = mem::size_of::<int>() as int;
                } else {
                    *E = mem::size_of::<u8>() as int;
                }
                E = E.offset(1);
                if TK == TokenKind::Inc as int {
                    *E = TokenKind::Sub as int;
                } else {
                    *E = TokenKind::Add as int;
                }
                next();
            } else if TK == TokenKind::Brak as int {
                next();
                E = E.offset(1);
                *E = OpCode::Psh as int;
                expr(TokenKind::Assign as int);
                if TK == b']' as int {
                    next();
                } else {
                    println!("{}: close bracket expected", LINE);
                    std::process::exit(-1);
                }
                if t > PTR {
                    E = E.offset(1);
                    *E = OpCode::Psh as int;
                    E = E.offset(1);
                    *E = OpCode::Imm as int;
                    E = E.offset(1);
                    *E = mem::size_of::<int>() as int;
                    E = E.offset(1);
                    *E = TokenKind::Mul as int;
                } else if t < PTR {
                    println!("{}: pointer type expected", LINE);
                    std::process::exit(-1);
                }
                E = E.offset(1);
                *E = TokenKind::Add as int;
                t = t - PTR;
                TY = t;
                E = E.offset(1);
                if t == CHAR {
                    *E = OpCode::Lc as int;
                } else {
                    *E = OpCode::Li as int;
                }
            } else {
                println!("{}: compiler error tk={}", LINE, TK);
                std::process::exit(-1);
            }
        }
    }
}

fn stmt() 
{
    unsafe {
        let mut a: *mut int;
        let mut b: *mut int;

        if TK == TokenKind::If as int {
            next();
            if TK == b'(' as int {
                next();
            } else {
                println!("{}: open paren expected", LINE);
                std::process::exit(-1);
            }
            expr(TokenKind::Assign as int);
            if TK == b')' as int {
                next();
            } else {
                println!("{}: close paren expected", LINE);
                std::process::exit(-1);
            }
            // *++e = BZ; b = ++e;
            E = E.offset(1);
            *E = OpCode::Bz as int;
            b = E.offset(1);
            E = E.offset(1);
            stmt();
            if TK == TokenKind::Else as int {
                // *b = (int)(e + 3); *++e = JMP; b = ++e;
                *b = (E.offset(3) as usize) as int;
                E = E.offset(1);
                *E = OpCode::Jmp as int;
                b = E.offset(1);
                E = E.offset(1);
                next();
                stmt();
            }
            *b = (E.offset(1) as usize) as int;
        } else if TK == TokenKind::While as int {
            next();
            a = E.offset(1);
            if TK == b'(' as int {
                next();
            } else {
                println!("{}: open paren expected", LINE);
                std::process::exit(-1);
            }
            expr(TokenKind::Assign as int);
            if TK == b')' as int {
                next();
            } else {
                println!("{}: close paren expected", LINE);
                std::process::exit(-1);
            }
            E = E.offset(1);
            *E = OpCode::Bz as int;
            b = E.offset(1);
            E = E.offset(1);
            stmt();
            E = E.offset(1);
            *E = OpCode::Jmp as int;
            E = E.offset(1);
            *E = a as int;
            *b = (E.offset(1) as usize) as int;
        } else if TK == TokenKind::Return as int {
            next();
            if TK != b';' as int {
                expr(TokenKind::Assign as int);
            }
            E = E.offset(1);
            *E = OpCode::Lev as int;
            if TK == b';' as int {
                next();
            } else {
                println!("{}: semicolon expected", LINE);
                std::process::exit(-1);
            }
        } else if TK == b'{' as int {
            next();
            while TK != b'}' as int {
                stmt();
            }
            next();
        } else if TK == b';' as int {
            next();
        } else {
            expr(TokenKind::Assign as int);
            if TK == b';' as int {
                next();
            } else {
                println!("{}: semicolon expected", LINE);
                std::process::exit(-1);
            }
        }
    }
}

fn main()
{
    let mut fd = -1;
    let mut bt = 0;
    let mut poolsz = POOLSZ;
    let mut idmain: *mut i32 = ptr::null_mut();
    let mut pc: *mut i32 = ptr::null_mut();
    let mut sp: *mut i32 = ptr::null_mut();
    let mut bp: *mut i32 = ptr::null_mut();
    let mut a: i32 = 0;
    let mut cycle: i32 = 0;
    let mut i: i32 = 0;
    let mut t: *mut i32 = ptr::null_mut();

    // Parse command-line arguments
    let args: Vec<String> = env::args().collect();
    let mut args_iter = args.iter();

    // Source mode
    if let Some(arg) = args_iter.next() {
        if arg == "-s" {
            src = 1;
        }
    }

    // Debug mode
    if let Some(arg) = args_iter.next() {
        if arg == "-d" {
            debug = 1;
        }
    }

    if args.len() < 2 {
        println!("usage: c4 [-s] [-d] file ...");
        return;
    }

    let filename = &args[1];

    // Open source file
    let file = File::open(filename).expect("could not open file");

    // Allocate memory for symbols, text, data, and stack
    unsafe {
        sym = malloc(poolsz);
        le = e = malloc(poolsz);
        data = malloc(poolsz);
        sp = malloc(poolsz);
    }

    if sym.is_null() || le.is_null() || data.is_null() || sp.is_null() {
        println!("could not malloc({})", poolsz);
        return;
    }

    // Memory area initialization
    unsafe {
        std::ptr::write_bytes(sym, 0, poolsz);
        std::ptr::write_bytes(e, 0, poolsz);
        std::ptr::write_bytes(data, 0, poolsz);
    }

    // Keywords
    p = "char else enum if int return sizeof while open read close printf malloc free memset memcmp exit void main".as_ptr() as *mut u8;
    i = Token::Char as i32;
    while i <= Token::While as i32 {
        next();
        unsafe { *id = i; }
        i += 1;
    }

    i = OpCode::Open as i32;
    while i <= OpCode::Exit
     as i32 {
        next();
        unsafe {
            *id = Token::Sys as i32;
            *id = VarType::INT as i32;
            *id = i;
        }
        i += 1;
    }

    next();
    unsafe { idmain = id; }

    // Load source code to memory
    unsafe {
        lp = p;
        let mut file_data = Vec::new();
        file.read_to_end(&mut file_data).expect("read() failed");
        p = file_data.as_ptr() as *mut u8;
        p[file_data.len()] = 0;
    }

    // Parse declarations
    line = 1;
    next();
    while tk != 0 {
        bt = VarType::INT as i32;

        if tk == Token::Int as i32 {
            next();
        } else if tk == Token::Char as i32 {
            next();
            bt = VarType::CHAR as i32;
        }

        // Enum initialization
        if tk == Token::Enum as i32 {
            next();
            if tk != '{' as i32 {
                next();
            }

            if tk == '{' as i32 {
                next();
                i = 0;
                while tk != '}' as i32 {
                    if tk != Token::Id as i32 {
                        println!("{}: bad enum identifier {}", line, tk);
                        return -1;
                    }
                    next();
                    if tk == Token::Assign as i32 {
                        next();
                        if tk != Token::Num as i32 {
                            println!("{}: bad enum initializer", line);
                            return -1;
                        }
                        i = ival;
                        next();
                    }
                    unsafe {
                        *id = Token::Num as i32;
                        *id = VarType::INT as i32;
                        *id = i;
                    }
                    if tk == ',' as i32 {
                        next();
                    }
                }
                next();
            }
        }

        // Global variables and functions
        while tk != ';' as i32 && tk != '}' as i32 {
            ty = bt;
            while tk == Token::Mul as i32 {
                next();
                ty += VarType::PTR as i32;
            }

            if tk != Token::Id as i32 {
                println!("{}: bad global declaration", line);
                return -1;
            }

            if unsafe { *id } != 0 {
                println!("{}: duplicate global definition", line);
                return -1;
            }

            next();
            unsafe { *id = ty };

            if tk == '(' as i32 {
                unsafe { *id = Token::Fun as i32 };
                unsafe { *id = (e + 1) as i32 };
                next();

                i = 0;
                while tk != ')' as i32 {
                    ty = VarType::INT as i32;
                    if tk == Token::Int as i32 {
                        next();
                    } else if tk == Token::Char as i32 {
                        next();
                        ty = VarType::CHAR as i32;
                    }
                    while tk == Token::Mul as i32 {
                        next();
                        ty += VarType::PTR as i32;
                    }
                    if tk != Token::Id as i32 {
                        println!("{}: bad parameter declaration", line);
                        return -1;
                    }
                    if unsafe { *id } == Token::Loc as i32 {
                        println!("{}: duplicate parameter definition", line);
                        return -1;
                    }
                    unsafe {
                        *id = Token::Loc as i32;
                        *id = ty;
                        *id = i;
                    }
                    next();
                    if tk == ',' as i32 {
                        next();
                    }
                }
                next();

                if tk != '{' as i32 {
                    println!("{}: bad function definition", line);
                    return;
                }

                loc = i + 1;
                next();
                while tk == Token::Int as i32 || tk == Token::Char as i32 {
                    bt = if tk == Token::Int as i32 { VarType::INT as i32 } else { VarType::CHAR as i32 };
                    next();
                    while tk != ';' as i32 {
                        ty = bt;
                        while tk == Token::Mul as i32 {
                            next();
                            ty += VarType::PTR as i32;
                        }
                        if tk != Token::Id as i32 {
                            println!("{}: bad local declaration", line);
                            return;
                        }
                        if unsafe { *id } == Token::Loc as i32 {
                            println!("{}: duplicate local definition", line);
                            return;
                        }
                        unsafe {
                            *id = Token::Loc as i32;
                            *id = ty;
                            *id = ++i;
                        }
                        next();
                        if tk == ',' as i32 {
                            next();
                        }
                    }
                    next();
                }

                unsafe {
                    *e = OpCode::Ent as i32;
                    *e = i - loc;
                }
                while tk != '}' as i32 {
                    stmt();
                }
                unsafe {
                    *e = OpCode::Lev
                    
                     as i32;
                }
                id = sym; // unwind symbol table locals
                while unsafe { *id } != 0 {
                    if unsafe { *id } == Token::Loc as i32 {
                        unsafe {
                            *id = Token::Glo as i32;
                        }
                    }
                    id = unsafe { id.add(7) };
                }
            } else {
                unsafe {
                    *id = Token::Glo as i32;
                    *id = data as i32;
                }
                data = unsafe { data.add(SIZEOF_INT) };
            }

            if tk == ',' as i32 {
                next();
            }
        }

        next();
    }

    if unsafe { *idmain } == 0 {
        println!("main() not defined");
        return;
    }

    if src != 0 {
        return;
    }

    // Setup stack
    bp = sp = unsafe { (sp.add(poolsz)) as *mut i32 };
    unsafe {
        *sp = OpCode::Exit as i32;
    }

    sp = sp.sub(1);
    unsafe {
        *sp = OpCode::Psh as i32;
    }
    t = sp;
    sp = sp.sub(1);
    unsafe {
        *sp = argc as i32;
    }
    sp = sp.sub(1);
    unsafe {
        *sp = argv as i32;
    }
    sp = sp.sub(1);
    unsafe {
        *sp = t as i32;
    }

    // Run the program
    cycle = 0;
    loop {
        i = unsafe { *pc };
        pc = pc.add(1);
        cycle += 1;

        if debug != 0 {
            // Debugging information here
        }

        match i {
            OpCode::Lea => a = unsafe { (bp.add(*pc)) as i32 },
            OpCode::Imm => a = unsafe { *pc },
            OpCode::Jmp => pc = unsafe { *pc as *mut i32 },
            OpCode::Jsr => {
                sp = sp.sub(1);
                *sp = (pc.add(1)) as i32;
                pc = unsafe { *pc as *mut i32 };
            },
            OpCode::Bz => {
                if a == 0 {
                    pc = unsafe { *pc as *mut i32 };
                }
            },
            OpCode::Bnz => {
                if a != 0 {
                    pc = unsafe { *pc as *mut i32 };
                }
            },
            OpCode::Ent => {
                sp = sp.sub(1);
                bp = sp;
                sp = sp.sub(*pc as usize);
            },
            OpCode::Adj => {
                sp = sp.add(*pc as usize);
            },
            OpCode::Lev => {
                sp = bp;
                bp = unsafe { *sp } as *mut i32;
                pc = unsafe { *sp } as *mut i32;
            },
            OpCode::Li => a = unsafe { *(a as *mut i32) },
            OpCode::Lc => a = unsafe { *(a as *mut u8) },
            OpCode::Si => {
                unsafe { *(sp as *mut i32) } = a;
            },
            OpCode::Sc => {
                unsafe { *(sp as *mut u8) } = a as u8;
            },
            OpCode::Psh => {
                sp = sp.sub(1);
                *sp = a;
            },
            OpCode::Or => {
                a = unsafe { *sp as i32 | a };
                sp = sp.add(1);
            },
            OpCode::Xor => {
                a = unsafe { *sp as i32 ^ a };
                sp = sp.add(1);
            },
            OpCode::And => {
                a = unsafe { *sp as i32 & a };
                sp = sp.add(1);
            },
            OpCode::Eq => {
                a = unsafe { *sp as i32 == a } as i32;
                sp = sp.add(1);
            },
            OpCode::Ne => {
                a = unsafe { *sp as i32 != a } as i32;
                sp = sp.add(1);
            },
            OpCode::Lt => {
                a = unsafe { *sp as i32 < a } as i32;
                sp = sp.add(1);
            },
            OpCode::Gt => {
                a = unsafe { *sp as i32 > a } as i32;
                sp = sp.add(1);
            },
            OpCode::Le => {
                a = unsafe { *sp as i32 <= a } as i32;
                sp = sp.add(1);
            },
            OpCode::Ge => {
                a = unsafe { *sp as i32 >= a } as i32;
                sp = sp.add(1);
            },
            OpCode::Shl => {
                a = unsafe { *sp as i32 << a } as i32;
                sp = sp.add(1);
            },
            OpCode::Shr => {
                a = unsafe { *sp as i32 >> a } as i32;
                sp = sp.add(1);
            },
            OpCode::Add => {
                a = unsafe { *sp as i32 + a };
                sp = sp.add(1);
            },
            OpCode::Sub => {
                a = unsafe { *sp as i32 - a };
                sp = sp.add(1);
            },
            OpCode::Mul => {
                a = unsafe { *sp as i32 * a };
                sp = sp.add(1);
            },
            OpCode::Div => {
                a = unsafe { *sp as i32 / a };
                sp = sp.add(1);
            },
            OpCode::Mod => {
                a = unsafe { *sp as i32 % a };
                sp = sp.add(1);
            },
            OpCode::Open => {
                a = unsafe { open(a as *const i8) };
                sp = sp.add(1);
            },
            OpCode::Read => {
                a = unsafe { read(a as i32, sp as *mut u8, 1024) as i32 };
                sp = sp.add(1);
            },
            OpCode::Clos => {
                a = unsafe { close(a as i32) };
                sp = sp.add(1);
            },
            OpCode::Prtf => {
                a = unsafe { printf(a as *const i8) };
                sp = sp.add(1);
            },
            OpCode::Malc => {
                a = unsafe { malloc(a as usize) };
                sp = sp.add(1);
            },
            OpCode::Free => {
                unsafe {
                    free(a as *mut u8);
                }
                sp = sp.add(1);
            },
            OpCode::Mset => {
                unsafe { memset(a as *mut u8, 0, a as usize) };
                sp = sp.add(1);
            },
            OpCode::Mcmp => {
                a = unsafe { memcmp(a as *const u8, sp as *const u8, 1024) };
                sp = sp.add(1);
            },
            OpCode::Exit => {
                break;
            },
        }
    }
    return;
}