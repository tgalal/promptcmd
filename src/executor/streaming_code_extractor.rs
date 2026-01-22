#[derive(Debug, Clone, PartialEq)]
pub(crate) enum ParserState {
    OutsideCode,
    PossibleFenceStart(String), // Accumulating backticks
    InFenceHeader(String),      // After ```, collecting language identifier
    InCode(String),             // Inside code block, collecting content
    PossibleFenceEnd(String, String), // Might be closing fence, (accumulated backticks, code so far)
}

pub struct StreamingCodeExtractor {
    state: ParserState,
    completed_blocks: Vec<String>,
}

impl StreamingCodeExtractor {
    pub fn new() -> Self {
        Self {
            state: ParserState::OutsideCode,
            completed_blocks: Vec::new(),
        }
    }

    pub fn feed(&mut self, token: &str, buffer: &mut String) -> bool {
        // self.curr_feed_buffer = String::new();
        for ch in token.chars() {
            self.process_char(ch, buffer);
        }
        matches!(self.state, ParserState::InCode(_) | ParserState::OutsideCode)
        // (&self.state, self.curr_feed_buffer.clone())
    }

    fn process_char(&mut self, ch: char, buffer: &mut String) {
        match &self.state {
            ParserState::OutsideCode => {
                if ch == '`' {
                    self.state = ParserState::PossibleFenceStart("`".to_string());
                }
            }

            ParserState::PossibleFenceStart(backticks) => {
                if ch == '`' {
                    let mut new_backticks = backticks.clone();
                    new_backticks.push('`');
                    if new_backticks.len() == 3 {
                        self.state = ParserState::InFenceHeader(String::new());
                    } else {
                        self.state = ParserState::PossibleFenceStart(new_backticks);
                    }
                } else {
                    // False alarm, not a fence
                    self.state = ParserState::OutsideCode;
                }
            }

            ParserState::InFenceHeader(lang) => {
                if ch == '\n' {
                    // Header complete, now collecting code
                    self.state = ParserState::InCode(String::new());
                } else {
                    let mut new_lang = lang.clone();
                    new_lang.push(ch);
                    self.state = ParserState::InFenceHeader(new_lang);
                }
            }

            ParserState::InCode(code) => {
                if ch == '`' {
                    // Might be closing fence
                    self.state = ParserState::PossibleFenceEnd("`".to_string(), code.clone());
                } else {
                    let mut new_code = code.clone();
                    new_code.push(ch);
                    buffer.push(ch);
                    self.state = ParserState::InCode(new_code);
                }
            }

            ParserState::PossibleFenceEnd(backticks, code) => {
                if ch == '`' {
                    let mut new_backticks = backticks.clone();
                    new_backticks.push('`');
                    if new_backticks.len() == 3 {
                        // Complete closing fence!
                        self.completed_blocks.push(code.clone());
                        self.state = ParserState::OutsideCode;
                    } else {
                        self.state = ParserState::PossibleFenceEnd(new_backticks, code.clone());
                    }
                } else {
                    // False alarm, those backticks were part of the code
                    let mut new_code = code.clone();
                    new_code.push_str(backticks);
                    new_code.push(ch);
                    buffer.push_str(backticks);
                    buffer.push(ch);
                    self.state = ParserState::InCode(new_code);
                }
            }
        }
    }
}
