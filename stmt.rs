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