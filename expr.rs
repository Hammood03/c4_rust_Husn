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