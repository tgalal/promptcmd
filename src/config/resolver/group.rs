use std::fmt;

use crate::config::resolver::{base::Base, variant::Variant};

#[derive(Debug)]
pub enum GroupMember {
    Base(Base, u32),
    Variant(Variant, u32)
}

impl GroupMember {
    pub fn weight(&self) -> u32 {
        match self {
            GroupMember::Base(_, weight) => *weight,
            GroupMember::Variant(_, weight) => *weight,
        }
    }
}

impl fmt::Display for GroupMember {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GroupMember::Base(base,_) => {
                write!(f, "{}", base)
            },
            GroupMember::Variant(variant,_) => {
                write!(f, "{}", variant)
            }
        }
    }
}

#[derive(Debug)]
pub struct Group {
    pub name: String,
    pub members: Vec<GroupMember>
}

impl fmt::Display for Group {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Group: {} =>", &self.name)?;
        for (i, member)  in self.members.iter().enumerate() {
            if i > 0 {
                writeln!(f)?;
            }
            let member_str = member.to_string();
            let lines: Vec<&str> = member_str.lines().collect();

            for (lineno, line) in lines.iter().enumerate() {
                if lineno == 0 {
                    write!(f, "- {line}")?;
                } else {
                    write!(f, "  {line}")?;
                }
                if lineno < lines.len() - 1 {
                    writeln!(f)?;
                }
            }
        }
        Ok(())
    }
}
