#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Not currently generating a block")]
    NoCurrentBlock,
}

pub struct Context {
    chunk: Vec<Stat>,
    current_blocks: Vec<Block>,
}

type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Clone, Debug)]
enum Block {
    Function(Function),
    Raw { code: Vec<Stat> },
}

#[derive(Clone, Debug)]
struct Function {
    name: Option<String>,
    params: Vec<String>,
    code: Vec<Stat>,
}

impl Function {
    fn render<W: std::fmt::Write>(&self, w: &mut W, ident: Ident) -> std::fmt::Result {
        write!(w, "function {}(", self.name.as_deref().unwrap_or(""))?;
        if let Some(p) = self.params.get(0) {
            write!(w, "{}", p)?;
        }
        if self.params.len() > 1 {
            for param in &self.params[1..] {
                write!(w, ",{}", param)?;
            }
        }
        writeln!(w, ")")?;

        let child_ident = ident.incr();
        for child in &self.code {
            child.render(w, child_ident)?;
        }

        writeln!(w, "{}end", ident)
    }
}

impl Block {
    fn append(&mut self, node: Stat) {
        match self {
            Block::Function(Function { code, .. }) => code.push(node),
            Block::Raw { code } => code.push(node),
        }
    }

    fn render<W: std::fmt::Write>(&self, w: &mut W, ident: Ident) -> std::fmt::Result {
        match self {
            Block::Function(f) => {
                write!(w, "{}local ", ident)?;
                f.render(w, ident)
            }
            Block::Raw { code } => {
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

// In lua all numbers are f64, but we carry ints in order to do
// constant propagation at a latter date
#[derive(Clone, Debug, Copy)]
pub enum Number {
    Int(i64),
    Float(f64),
}

#[derive(Clone, Debug)]
enum Var {
    Ident(String),
    Expression(Expression),
}

#[derive(Clone, Debug)]
pub struct Place(Var);

impl Var {
    fn render<W: std::fmt::Write>(&self, w: &mut W, ident: Ident) -> std::fmt::Result {
        match self {
            Var::Ident(i) => write!(w, "{}", i),
            Var::Expression(e) => {
                write!(w, "(")?;
                e.render(w, ident)?;
                write!(w, ")")
            }
        }
    }
}

pub struct Call {
    function: Var,
    parameters: Vec<Expression>,
}

#[derive(Clone, Debug)]
enum Expression {
    //Function(Function),
    Number(Number),
    Ident(String),
    TableAccess {
        table: Box<Var>,
        key: Box<Expression>,
    },
    Nil,
    Table(Vec<(Expression, Expression)>),
    String(String),
}

#[derive(Clone, Debug)]
pub struct Value(Expression);

impl Expression {
    fn render<W: std::fmt::Write>(&self, w: &mut W, ident: Ident) -> std::fmt::Result {
        match self {
            //Expression::Function(f) => f.render(w, ident),
            Expression::Number(Number::Float(v)) => write!(w, "{}", v),
            Expression::Number(Number::Int(v)) => write!(w, "{}", v),
            Expression::Ident(name) => write!(w, "{}", name),
            Expression::Nil => write!(w, "nil"),
            Expression::TableAccess { table, key } => {
                write!(w, "(")?;
                table.render(w, ident)?;
                write!(w, ")[")?;
                key.render(w, ident)?;
                write!(w, "]")
            }
            Expression::Table(fields) => {
                write!(w, "{{")?;
                if let Some((k, v)) = fields.first() {
                    write!(w, "[")?;
                    k.render(w, ident)?;
                    write!(w, "] = (")?;
                    v.render(w, ident)?;
                    write!(w, ")")?;
                }
                if fields.len() > 1 {
                    for (k, v) in &fields[1..] {
                        write!(w, ",[")?;
                        k.render(w, ident)?;
                        write!(w, "] = (")?;
                        v.render(w, ident)?;
                        write!(w, ")")?;
                    }
                }
                write!(w, "}}")
            }
            Expression::String(v) => write!(w, "\"{}\"", v),
        }
    }

    fn to_place(self) -> Place {
        Place(Var::Expression(self))
    }
}

impl Value {
    #[inline]
    pub fn to_place(self) -> Place {
        self.0.to_place()
    }
}

#[derive(Clone, Debug)]
enum Stat {
    Block(Block),
    Assign { place: Var, value: Expression },
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

impl Stat {
    fn render<W: std::fmt::Write>(&self, w: &mut W, ident: Ident) -> std::fmt::Result {
        match self {
            Stat::Block(b) => b.render(w, ident),
            Stat::Assign { place, value } => {
                write!(w, "{}local ", ident)?;
                place.render(w, ident)?;
                write!(w, " = ")?;
                value.render(w, ident)?;
                writeln!(w, ";")
            }
        }
    }
}

pub struct ExprBuilder;

impl ExprBuilder {
    pub fn nil(self) -> Value {
        Value(Expression::Nil)
    }

    pub fn int(self, value: i64) -> Value {
        Value(Expression::Number(Number::Int(value)))
    }

    pub fn double(self, value: f64) -> Value {
        Value(Expression::Number(Number::Float(value)))
    }

    pub fn get_place(self, place: Place) -> Value {
        match place.0 {
            Var::Ident(x) => Value(Expression::Ident(x)),
            Var::Expression(e) => Value(e),
        }
    }

    pub fn table_access(self, table: Place, key: Value) -> Value {
        Value(Expression::TableAccess {
            key: Box::new(key.0),
            table: Box::new(table.0),
        })
    }

    pub fn table<I: IntoIterator<Item = (Value, Value)>>(self, values: I) -> Value {
        Value(Expression::Table(
            values.into_iter().map(|(k, v)| (k.0, v.0)).collect(),
        ))
    }

    pub fn string(self, value: String) -> Value {
        Value(Expression::String(value))
    }
}

pub struct StatBuilder<'ctx> {
    ctx: &'ctx mut Context,
}

impl<'ctx> StatBuilder<'ctx> {
    pub fn assign(self, place: Place, value: Value) {
        let stat = Stat::Assign {
            place: place.0,
            value: value.0,
        };
        self.ctx.add_stat(stat)
    }
}

impl Context {
    pub fn new() -> Self {
        Context {
            current_blocks: Vec::new(),
            chunk: Vec::new(),
        }
    }

    fn add_stat(&mut self, stat: Stat) {
        match self.current_blocks.last_mut() {
            Some(b) => b.append(stat),
            None => self.chunk.push(stat),
        }
    }

    pub fn start_function(&mut self, name: String, params: Vec<String>) -> Place {
        self.current_blocks.push(Block::Function(Function {
            name: Some(name.clone()),
            params,
            code: Vec::new(),
        }));

        Place(Var::Ident(name))
    }

    pub fn start_raw_block(&mut self) {
        self.current_blocks.push(Block::Raw { code: Vec::new() });
    }

    pub fn finish_block(&mut self) -> Result<()> {
        let finished_block = match self.current_blocks.pop() {
            Some(b) => b,
            None => return Err(Error::NoCurrentBlock),
        };
        self.add_stat(Stat::Block(finished_block));

        Ok(())
    }

    pub fn stat(&mut self) -> StatBuilder<'_> {
        StatBuilder { ctx: self }
    }

    pub fn expr(&mut self) -> ExprBuilder {
        ExprBuilder
    }

    pub fn declare(&mut self, name: String) -> Place {
        Place(Var::Ident(name))
    }

    pub fn render<W: std::fmt::Write>(&self, w: &mut W) -> std::fmt::Result {
        for block in &self.chunk {
            block.render(w, Ident(0))?;
        }

        Ok(())
    }
}
