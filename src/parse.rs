use crate::lex::Token;
use crate::typecheck::{Type, Primative, Mutability, Fn as FnType};

#[derive(Debug)]
pub struct Program<'a>{
    // todo this should probably be an array of Box<ParsedAST>
    pub body: Vec<ParsedAST<'a>>
}

#[derive(Debug)]
pub struct Block<'a>{
    pub body: Vec<ParsedAST<'a>>
}

#[derive(Debug)]
pub struct Decl<'a>{
    pub identifier: &'a Token,
    pub typ: Type,
    pub requires_infering: bool,
    pub value: Box<ParsedAST<'a>>,
}

#[derive(Debug)]
pub struct Identifier<'a> {
    pub token: &'a Token
}

#[derive(Debug)]
pub struct Binary<'a>{
    pub left: Box<ParsedAST<'a>>,
    pub op: &'a Token, // todo this should probably be a ref
    pub right: Box<ParsedAST<'a>>
}

#[derive(Debug)]
pub struct Group<'a>{
    pub expression: Box<ParsedAST<'a>>
}

#[derive(Debug)]
pub struct Call<'a>{
    pub callee: Box<ParsedAST<'a>>,
    pub args: Box<ParsedAST<'a>>
}

#[derive(Debug)]
pub struct Fn<'a>{
    pub body: Box<ParsedAST<'a>>
}


#[derive(Debug)]
pub enum ParsedAST<'a> {
    PROGRAM(Program<'a>),
    STMT(Box<ParsedAST<'a>>),
    BLOCK(Block<'a>),
    DECL(Decl<'a>),
    IDENTIFIER(std::string::String),
    STRING(std::string::String),
    FN(Fn<'a>),
    NUMBER(f32),
    BINARY(Binary<'a>),
    GROUP(Group<'a>),
    CALL(Call<'a>),
}

pub struct Parser<'a>{
    pub tokens: &'a Box<Vec<Token>>
}
// todo should the ast be an enum?


impl Parser<'_> {

    pub fn new(tokens: &mut Box<Vec<Token>>) -> Parser {
        Parser {tokens}
    }

    pub fn parse(&mut self) -> Box<ParsedAST> {
        Box::new(self.parse_program())
    }

    fn parse_program(&mut self) -> ParsedAST{
        println!("parsing program!");

        let mut current: usize = 0;
        let mut body: Vec<ParsedAST> = vec!();
        
        while !self.end(&current){
            body.push(self.statement(&mut current));
        }
        
        return ParsedAST::PROGRAM(Program{body: body});
    }

    fn parse_type(&self, current: &mut usize) -> Type {
        match self.consume(current) {
            Token::U32 => Type{mutability: Mutability::CONSTANT, primative: Primative::U32},
            Token::I32 => Type{mutability: Mutability::CONSTANT, primative: Primative::I32},
            Token::BOOL => Type{mutability: Mutability::CONSTANT, primative: Primative::BOOL},
            Token::FN => Type{mutability: Mutability::CONSTANT, primative: Primative::FN(FnType{args: vec![]})},
            Token::TYPE => Type{mutability: Mutability::CONSTANT, primative: Primative::TYPE},
            _ => panic!()
        }

    }

    fn statement(&self, current: &mut usize) -> ParsedAST {
        match self.peek(&current) {
            Token::LCURLY => self.block(current),
            _ => ParsedAST::STMT(Box::new(self.expression(current)))
        }
    }

    fn expression(&self, current: &mut usize) -> ParsedAST {
        match self.peek(&current) {
            _ => self.comparison(current)
        }
    }

    fn comparison(&self, current: &mut usize) -> ParsedAST {
        self.decl_or_assign(current)
    }

    fn decl_or_assign(&self, current: &mut usize) -> ParsedAST {

        let first = self.peek(current);

        if self.end_ahead(current, 1) {
            return self.assign(current);
        }

        let second = self.peek_ahead(current, 1);

        match first {
            Token::IDENTIFIER(_) => {
                match second {
                    // todo we need to match for a type here instead of identifier
                    Token::U32 | Token::I32 | Token::BOOL | Token::TYPE | Token::FN => {
                        let identifier = self.consume(current);
                        let typ = self.parse_type(current);

                        let x = self.peek(current);

                        // constant
                        match self.peek(current) {
                            Token::EQUAL => {
                                self.consume(current); // consume the =
                            },
                            _ => {}
                        };

                        let value = self.expression(current);
                        return ParsedAST::DECL(Decl{identifier, typ, requires_infering: false, value: Box::new(value)})
                    },
                    Token::EQUAL => {
                        let identifier = self.consume(current);
                        let typ = Type{mutability: Mutability::CONSTANT, primative: Primative::NONE};
                        self.consume(current); // consume the =
                        let value = self.expression(current);
                        return ParsedAST::DECL(Decl{identifier, typ, requires_infering: true, value: Box::new(value)})

                    },
                    _ => return self.assign(current)
                }
            },
            _ => return self.assign(current)
        }
    }

    fn assign(&self, current: &mut usize) -> ParsedAST {
        self.plus_or_minus(current)
    }

    fn plus_or_minus(&self, current: &mut usize) -> ParsedAST {
        let higher_precedence = self.mul_or_div(current);

        if !self.end(current){
            match self.peek(current) {
                Token::PLUS | Token::MINUS => {
                    let token = self.consume(current);
                    let right = self.expression(current);
                    return ParsedAST::BINARY(Binary{
                        left: Box::new(higher_precedence),
                        op: token,
                        right: Box::new(right)
                    })
                },
                _ => return higher_precedence
            }
        }
        higher_precedence
    }

    fn mul_or_div(&self, current: &mut usize) -> ParsedAST {
        let higher_precedence = self.unary(current);

        if !self.end(current){
            match self.peek(current) {
                Token::STAR | Token::DIV => {
                    let token = self.consume(current);
                    let right = self.expression(current);
                    return ParsedAST::BINARY(Binary{
                        left: Box::new(higher_precedence),
                        op: token,
                        right: Box::new(right)
                    })
                },
                _ => return higher_precedence
            }
        }
        higher_precedence
    }

    fn unary(&self, current: &mut usize) -> ParsedAST {
        self.call(current)
    }

    fn call(&self, current: &mut usize) -> ParsedAST {
        let higher_presedence = self.struct_access(current);
        if !self.end(current) {
            match self.peek_ahead(current, -1) {
                Token::IDENTIFIER(_) => {
                    if !self.end_ahead(current, 1){
                        // todo
                        // todo peak_ahead could fail :(
                        match self.peek(current) {
                            Token::LPAREN => {
                                self.consume(current);
                                // todo get args
                                let expr = self.expression(current);
                                self.consume(current);
                                return ParsedAST::CALL(Call{callee: Box::new(higher_presedence), args: Box::new(expr)})
                            }
                            _ => return higher_presedence
                        }
                    }
                }
                _ => return higher_presedence
            }
        }

        higher_presedence
    }

    fn struct_access(&self, current: &mut usize) -> ParsedAST {
        self.single(current)
    }

    fn single(&self, current: &mut usize) -> ParsedAST {
        match self.peek(current) {
            Token::TRUE => {
                self.consume(current);
                ParsedAST::NUMBER(1.0)
            },
            Token::FALSE => {
                self.consume(current);
                ParsedAST::NUMBER(0.0)
            },
            Token::IDENTIFIER(identifier) => {
                self.consume(current);
                ParsedAST::IDENTIFIER(identifier.to_string())
            },
            Token::STRING(string) => {
                self.consume(current);
                ParsedAST::STRING(string.to_string())
            },
            Token::NUMBER(number) => {
                self.consume(current);
                match number.parse::<f32>(){
                    Ok(num) => ParsedAST::NUMBER(num),
                    _ => panic!()
                }
            },
            Token::LPAREN => {
                self.consume(current);
                let expression = self.expression(current);
                self.consume(current);
                ParsedAST::GROUP(Group { expression: Box::new(expression) })
            },
            Token::LCURLY => {
                self.block(current)
            }
            _ => panic!()
        }
    }

    fn peek(&self, current: &usize) -> &Token {
        match self.tokens.get(*current) {
            std::option::Option::Some(t) => return t,
            _ => panic!("umm")
        }
    }

    fn peek_ahead(&self, current: &usize, amount: i32) -> &Token {
        match self.tokens.get((*current as i32 + amount) as usize) {
            std::option::Option::Some(t) => return t,
            _ => panic!("umm")
        }
    }

    fn end(&self, current: &usize) -> bool {
        *current >= self.tokens.len()
    }

    fn end_ahead(&self, current: &usize, amount: i32) -> bool {
        (*current as i32 + amount) as usize >= self.tokens.len()
    }

    fn expecting(&self, token: Token, current: &usize) -> bool {
        let next = self.peek(current);
        return token.eq(&next);
    }

    fn consume(&self, current: &mut usize) -> &Token {
        match self.tokens.get(*current) {
            std::option::Option::Some(t) => {
                *current+=1;
                return t;
            },
            _ => panic!("umm")
        }
    }

    fn block(&self, current: &mut usize) -> ParsedAST {
        self.consume(current);
        let mut body: Vec<ParsedAST> = vec!();
        while !self.end(current) && !self.expecting(Token::RCURLY, current) {
            body.push(self.statement(current));
        }
        self.consume(current);
        return ParsedAST::BLOCK(Block{body});
    }
}