use crate::err::TokErr;

pub struct Peeker<T: Clone, I: Iterator<Item = T>> {
    it: I,
    peekv: Option<T>,
}

impl<T: Clone, I: Iterator<Item = T>> Iterator for Peeker<T, I> {
    type Item = T;
    fn next(&mut self) -> Option<T> {
        match self.peekv {
            Some(_) => self.peekv.take(),
            None => self.it.next(),
        }
    }
}
impl<'a> Peeker<char, std::str::Chars<'a>> {
    pub fn from_str(s: &'a str) -> Self {
        Peeker {
            it: s.chars(),
            peekv: None,
        }
    }
}

impl<T: Clone, I: Iterator<Item = T>> Peeker<T, I> {
    pub fn from_iter(it: I) -> Self {
        Peeker { it, peekv: None }
    }
    pub fn peek(&mut self) -> Option<T> {
        if let None = self.peekv {
            self.peekv = self.it.next();
        }
        self.peekv.clone()
    }

    pub fn find_before<F>(&mut self, mut f: F) -> Option<T>
    where
        F: FnMut(T) -> bool,
    {
        while let Some(r) = self.peek() {
            if f(r.clone()) {
                return Some(r);
            }
            self.next();
        }
        None
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    Break,
    UScore,
    DUScore,
    Dash,
    Slash,
    Equals,
    Colon,
    Num(i32),
    Str(String),
}

impl Token {
    pub fn as_num(&self) -> Result<i32, TokErr> {
        if let Num(n) = *self {
            return Ok(n);
        }
        Err(TokErr::NotNum(self.clone()))
    }
    pub fn as_str(&self) -> Result<String, TokErr> {
        if let Str(s) = self {
            return Ok(s.clone());
        }
        Err(TokErr::NotString(self.clone()))
    }
}

use self::Token::*;

pub struct Tokeniser<'a> {
    it: Peeker<char, std::str::Chars<'a>>,
}

impl<'a> Tokeniser<'a> {
    pub fn new(s: &'a str) -> Self {
        Tokeniser {
            it: Peeker::from_str(s),
        }
    }
}

impl Iterator for Tokeniser<'_> {
    type Item = Token;
    fn next(&mut self) -> Option<Token> {
        match self.it.next() {
            None => None,
            Some(',') | Some('\n') => Some(Break),
            Some(':') => Some(Colon),
            Some('#') => {
                self.it.find(|x| *x == ',' || *x == '\n');
                Some(Break)
            }
            Some('_') => {
                if let Some('_') = self.it.peek() {
                    self.it.next();
                    Some(DUScore)
                } else {
                    Some(UScore)
                }
            }
            Some('=') => Some(Equals),
            Some('-') => Some(Dash),
            Some('/') => Some(Slash),
            Some('\t') | Some(' ') => self.next(),
            Some(c) => {
                if c >= '0' && c <= '9' {
                    let mut res = c as i32 - 48;
                    loop {
                        match self.it.peek() {
                            Some(nx) => {
                                if nx < '0' || nx > '9' {
                                    return Some(Num(res));
                                }
                                res = res * 10 + (nx as i32) - 48;
                            }
                            None => return Some(Num(res)),
                        }
                        self.it.next();
                    }
                } else {
                    let mut res = c.to_string();
                    loop {
                        match self.it.peek() {
                            None | Some(',') | Some(':') | Some('_') | Some('\n') => {
                                return Some(Str(res))
                            }
                            Some(c) => {
                                self.it.next();
                                res.push(c);
                            }
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    pub fn test_peeker() {
        let s = "123456789";
        let p = Peeker::from_str(s);
        let c: String = p.collect();
        assert_eq!(&c, "123456789");

        let mut p2 = Peeker::from_str(s);

        assert_eq!(p2.next(), Some('1'));
        assert_eq!(p2.peek(), Some('2'));

        assert_eq!(p2.next(), Some('2'));
        assert_eq!(p2.peek(), Some('3'));
        assert_eq!(p2.peek(), Some('3'));
        assert_eq!(p2.peek(), Some('3'));
        assert_eq!(p2.next(), Some('3'));
    }

    #[test]
    pub fn test_tokens() {
        let s = "hello, #poop ,every body,34ghosts";
        let c: Vec<Token> = Tokeniser::new(s).collect();
        assert_eq!(
            c,
            vec![
                Str("hello".to_string()),
                Break,
                Break,
                Str("every body".to_string()),
                Break,
                Num(34),
                Str("ghosts".to_string()),
            ]
        );
    }
}
