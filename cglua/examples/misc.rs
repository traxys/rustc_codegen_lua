use cglua::Context;

pub fn main() {
    let mut ctx = Context::new();

    ctx.start_function("foo".into(), Vec::new());
    {
        ctx.start_function("bar".into(), vec!["a".into(), "b".into()]);
        {
            ctx.start_block();
            {}
            ctx.finish_blk().unwrap();
            ctx.start_function("rat".into(), vec!["a".into(), "b".into()]);
            {}
            ctx.finish_blk().unwrap();
        }
        ctx.finish_blk().unwrap();
    }
    ctx.finish_blk().unwrap();

    let mut s = String::new();
    ctx.render(&mut s).unwrap();
    println!("{}", s);
}
