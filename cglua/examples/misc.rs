use cglua::Context;

pub fn main() {
    let mut ctx = Context::new();

    ctx.start_function("foo".into(), Vec::new());
    {
        ctx.start_function("bar".into(), vec!["a".into(), "b".into()]);
        {
            ctx.start_raw_block();
            {
                let x = ctx.declare("x".to_string());
                let nil = ctx.expr().nil();
                ctx.stat().assign(x, nil);
            }
            ctx.finish_block().unwrap();
            ctx.start_function("rat".into(), vec!["a".into(), "b".into()]);
            {
                let x = ctx.declare("x".to_string());
                let ft = ctx.expr().int(42);
                let ft_f = ctx.expr().double(42.5);
                ctx.stat().assign(x.clone(), ft);
                ctx.stat().assign(x, ft_f);
            }
            ctx.finish_block().unwrap();
        }
        ctx.finish_block().unwrap();
        let test = ctx.expr().string("test".into());
        let one = ctx.expr().int(1);
        let two = ctx.expr().int(2);
        let table = ctx.expr().table([(test, one.clone()), (one, two)]);
        let x = ctx.declare("x".into());
        ctx.stat().assign(x, table);
    }
    ctx.finish_block().unwrap();

    let mut s = String::new();
    ctx.render(&mut s).unwrap();
    println!("{}", s);
}
