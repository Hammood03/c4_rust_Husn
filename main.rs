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