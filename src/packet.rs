use rand::Rng;
use std::fmt::Display;
use std::net::Ipv4Addr;

use crate::deserialization::{pop_collection, pop_u16, FromBytes};
use crate::domain_name::DomainName;
use crate::record::Record;
use crate::record::{Class, Kind};
use crate::serialization::push_u16;



#[derive(Default, Clone, Copy)]
pub struct Flags(u16);

impl Flags {
    pub fn new() -> Flags {
        Flags(0)
    }
    pub fn with_recusion(mut self) -> Flags {
        self.0 |= 1 << 8;
        self
    }
}

#[derive(Debug, Default, Clone)]
pub struct Packet {
    pub id: u16,
    pub flags: u16,
    pub questions: Vec<Question>,
    pub answers: Vec<Record>,
    pub authorities: Vec<Record>,
    pub additionals: Vec<Record>,
}

impl Packet {
    pub fn new() -> Packet {
        let id = rand::thread_rng().gen();
        let flags = 0;
        Packet {
            id,
            flags,
            questions: vec![],
            answers: vec![],
            authorities: vec![],
            additionals: vec![],
        }
    }

    pub fn with_flags(mut self, flags: Flags) -> Packet {
        self.flags = flags.0;
        self
    }
    pub fn with_question(mut self, question: Question) -> Packet {
        self.questions.push(question);
        self
    }
    // pub fn build(domain: &str, kind: Kind) -> Packet {
    //     let q = Question::build(domain, kind);
    //     let id = rand::thread_rng().gen();
    //     let flags = 0;
    //     Packet {
    //         id,
    //         flags,
    //         questions: vec![q],
    //         answers: vec![],
    //         authorities: vec![],
    //         additionals: vec![],
    //     }
    // }
    pub fn with_id(mut self, id: u16) -> Packet {
        self.id = id;
        self
    }
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        push_u16(&mut buf, self.id);
        push_u16(&mut buf, self.flags);
        push_u16(&mut buf, self.questions.len() as u16);
        push_u16(&mut buf, 0);
        push_u16(&mut buf, 0);
        push_u16(&mut buf, 0);

        for question in self.questions.iter() {
            buf.extend_from_slice(&question.to_bytes());
        }
        buf
    }
    pub fn from_bytes(buf: &[u8]) -> Option<Packet> {
        let mut cursor = 0;
        let header = Header::from_bytes(buf, &mut cursor)?;

        let Header {
            id,
            flags,
            questions,
            answers,
            authorities,
            additionals,
        } = header;
        let questions = pop_collection(buf, &mut cursor, questions as usize)?;
        let answers = pop_collection(buf, &mut cursor, answers as usize)?;
        let authorities = pop_collection(buf, &mut cursor, authorities as usize)?;
        let additionals = pop_collection(buf, &mut cursor, additionals as usize)?;

        Some(Packet {
            id,
            flags,
            questions,
            answers,
            authorities,
            additionals,
        })
    }
}

fn flag_write(
    f: &mut std::fmt::Formatter<'_>,
    flags: &u16,
    offset: usize,
    zero_label: &str,
    one_label: &str,
) -> std::fmt::Result {
    let label = if (flags & (0b1 << offset)) == (0b1 << offset) {
        one_label
    } else {
        zero_label
    };
    write!(f, "{label}")
}

impl Display for Packet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Packet#{:x} (", self.id)?;
        {
            flag_write(f, &self.flags, 0, "Q-", "R-")?;
            // if  (self.flags & (1 << 0)) == (1 << 0) {
            //     write!(f, "Qr ")?;
            // } else {
            //     write!(f, "qR ")?;
            // }

            let opcode = (self.flags >> 1) & 0b1111;
            write!(f, "{opcode:02x}-")?;

            flag_write(f, &self.flags, 5, "aa-", "AA-")?;
            flag_write(f, &self.flags, 6, "tc-", "TC-")?;
            flag_write(f, &self.flags, 7, "rd-", "RD-")?;
            flag_write(f, &self.flags, 8, "ra-", "RA-")?;
            flag_write(f, &self.flags, 9, "z-", "Z-")?;
            flag_write(f, &self.flags, 10, "ad-", "AD-")?;
            flag_write(f, &self.flags, 11, "cd-", "CD-")?;

            let rcode = (self.flags >> 12) & 0b1111;
            write!(f, "{rcode:02x}")?;

            writeln!(f, ")")?;
        }
        if self.questions.is_empty()
            && self.answers.is_empty()
            && self.authorities.is_empty()
            && self.additionals.is_empty()
        {
            write!(f, "Empty Packet")?;
        }
        if !self.questions.is_empty() {
            writeln!(f, "\tQuestions: {}", self.questions.len())?;
            for q in self.questions.iter() {
                writeln!(f, "\t\t{}", q)?;
            }
        }
        if !self.answers.is_empty() {
            writeln!(f, "\tAnswers: {}", self.answers.len())?;
            for q in self.answers.iter() {
                writeln!(f, "\t\t{}", q)?;
            }
        }
        if !self.authorities.is_empty() {
            writeln!(f, "\tAuthorities: {}", self.authorities.len())?;
            for q in self.authorities.iter() {
                writeln!(f, "\t\t{}", q)?;
            }
        }
        if !self.additionals.is_empty() {
            writeln!(f, "\tAdditionals: {}", self.additionals.len())?;
            for q in self.additionals.iter() {
                writeln!(f, "\t\t{}", q)?;
            }
        }
        writeln!(f)
    }
}

#[derive(Debug)]
pub struct Header {
    id: u16,
    flags: u16,
    questions: u16,
    answers: u16,
    authorities: u16,
    additionals: u16,
}

impl Header {
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(12);
        push_u16(&mut buf, self.id);
        push_u16(&mut buf, self.flags);
        push_u16(&mut buf, self.questions);
        push_u16(&mut buf, self.answers);
        push_u16(&mut buf, self.authorities);
        push_u16(&mut buf, self.additionals);
        buf
    }
    pub fn from_bytes(buf: &[u8], cursor: &mut usize) -> Option<Header> {
        let id = pop_u16(buf, cursor)?;
        let flags = pop_u16(buf, cursor)?;
        let questions = pop_u16(buf, cursor)?;
        let answers = pop_u16(buf, cursor)?;
        let authorities = pop_u16(buf, cursor)?;
        let additionals = pop_u16(buf, cursor)?;
        Some(Header {
            id,
            flags,
            questions,
            answers,
            authorities,
            additionals,
        })
    }
}

#[derive(Debug, Clone, Default)]
pub struct Question {
    name: DomainName,
    kind: Kind,
    class: Class,
}

impl Question {
    pub fn new() -> Question {
        Question {
            name: DomainName::empty(),
            kind: Kind::A,
            class: Class::Internet,
        }
    }
    pub fn with_domain_name(mut self, name: &str) -> Question {
        self.name = DomainName::new(name);
        self
    }
    pub fn with_kind(mut self, kind: Kind) -> Question {
        self.kind = kind;
        self
    }
    pub fn build(name: &str, kind: Kind) -> Question {
        let name = DomainName::new(name);
        Question {
            name,
            kind,
            class: Class::Internet,
        }
    }
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buf: Vec<u8> = self.name.to_bytes();
        push_u16(&mut buf, self.kind as u16);
        push_u16(&mut buf, self.class as u16);
        buf
    }
}

impl Display for Question {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        <DomainName as Display>::fmt(&self.name, f)?;
        write!(f, " ")?;
        <Kind as Display>::fmt(&self.kind, f)?;
        write!(f, " ")?;
        <Class as Display>::fmt(&self.class, f)?;
        write!(f, " ")
    }
}

impl FromBytes for Question {
    fn from_bytes(buf: &[u8], cursor: &mut usize) -> Option<Self> {
        let name = DomainName::from_bytes(buf, cursor)?;
        let kind = Kind::from_bytes(buf, cursor)?;
        let class = Class::from_bytes(buf, cursor)?;
        Some(Question { name, kind, class })
    }
}
