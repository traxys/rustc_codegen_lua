pub struct Context {
    root: Vec<Node>,
    current_blocks: Vec<Block>,
}

type Result<T, E = Error> = std::result::Result<T, E>;

enum Block {
    Function {
        name: String,
        params: Vec<String>,
        code: Vec<Node>,
    },
    Block {
        code: Vec<Node>,
    },
}

impl Block {
    fn append(&mut self, node: Node) {
        match self {
            Block::Function { code, .. } => code.push(node),
            Block::Block { code } => code.push(node),
        }
    }

    fn render<W: std::fmt::Write>(&self, w: &mut W, ident: Ident) -> std::fmt::Result {
        match self {
            Block::Function { name, params, code } => {
                write!(w, "{}function {}(", ident, name)?;
                if let Some(p) = params.get(0) {
                    write!(w, "{}", p)?;
                }
                if params.len() > 1 {
                    for param in &params[1..] {
                        write!(w, ",{}", param)?;
                    }
                }
                writeln!(w, ")")?;

                let child_ident = ident.incr();
                for child in code {
                    child.render(w, child_ident)?;
                }

                writeln!(w, "{}end", ident)
            }
            Block::Block { code } => {
                writeln!(w, "{}do", ident)?;
                let child_ident = ident.incr();
                for child in code {
                    child.render(w, child_ident)?;
                }
                writeln!(w, "{}end", ident)
            }
        }
    }
}

pub struct Number(f64);

enum Node {
    Block(Block),
}

#[derive(Clone, Copy)]
struct Ident(usize);

impl std::fmt::Display for Ident {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:width$}", "", width = self.0 * 2)
    }
}

impl Ident {
    fn incr(&self) -> Self {
        Self(self.0 + 1)
    }
}

impl Node {
    fn render<W: std::fmt::Write>(&self, w: &mut W, ident: Ident) -> std::fmt::Result {
        match self {
            Node::Block(b) => b.render(w, ident),
        }
    }
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Not currently generating a block")]
    NoCurrentBlock,
}

impl Context {
    pub fn new() -> Self {
        Context {
            current_blocks: Vec::new(),
            root: Vec::new(),
        }
    }

    pub fn start_function(&mut self, name: String, params: Vec<String>) {
        self.current_blocks.push(Block::Function {
            name,
            params,
            code: Vec::new(),
        });
    }

    pub fn start_block(&mut self) {
        self.current_blocks.push(Block::Block { code: Vec::new() });
    }

    pub fn finish_blk(&mut self) -> Result<()> {
        let finished_block = match self.current_blocks.pop() {
            Some(b) => b,
            None => return Err(Error::NoCurrentBlock),
        };
        match self.current_blocks.last_mut() {
            Some(b) => b.append(Node::Block(finished_block)),
            None => self.root.push(Node::Block(finished_block)),
        }

        Ok(())
    }

    pub fn render<W: std::fmt::Write>(&self, w: &mut W) -> std::fmt::Result {
        for block in &self.root {
            block.render(w, Ident(0))?;
        }

        Ok(())
    }
}
