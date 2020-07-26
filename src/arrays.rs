use regex::Regex;
use std::fmt::Write;

fn array_dim_expansion(n: usize) -> String {
    let mut buffer = String::new();
    for i in 0..n {
        writeln!(&mut buffer, "dim[{}],", i).expect("Write failed!");
    }
    buffer
}

fn ctor_array_type(t: &str, dim: usize) -> String {
    format!("Array_{}_{}d", t, dim)
}

fn gen_specific_type(input: &str) -> String {
    let re_array_type = Regex::new(r"futhark_(.+)_(\d+)d").unwrap();
    let captures = re_array_type.captures(input).unwrap();
    let dim: usize = captures[2].parse().unwrap();
    let ftype = &captures[1];
    let arr_type = ctor_array_type(ftype, dim);
    let oftype = format!("futhark_{}_{}d", ftype, dim);
    format!(
        include_str!("static/static_array_types.rs"),
        array_type = arr_type,
        futhark_type = oftype,
        dim = dim,
        inner_type = ftype
    )
}

fn gen_impl_futhark_type(input: &str) -> String {
    let re_array_type = Regex::new(r"futhark_(.+)_(\d+)d").unwrap();
    let captures = re_array_type.captures(input).unwrap();
    let dim: usize = captures[2].parse().unwrap();
    let mut buffer = String::new();
    let arr_type = ctor_array_type(&captures[1], dim);
    write!(&mut buffer,
r#"
impl futhark_{rust_type}_{dim}d {{
   unsafe fn new(ctx: &crate::context::FutharkContext, arr: &[{rust_type}], dim: &[i64]) -> *const Self
   {{
     let ctx = ctx.ptr();
     bindings::futhark_new_{rust_type}_{dim}d(
       ctx,
       arr.as_ptr() as *mut {rust_type},
       {array_dim})
     }}
}}

impl FutharkType for futhark_{rust_type}_{dim}d {{
   type RustType = {rust_type};
   const DIM: usize = {dim};

    unsafe fn shape(ctx: &crate::context::FutharkContext, ptr: *const bindings::futhark_{rust_type}_{dim}d) -> *const i64
    {{
        let ctx = ctx.ptr();
        bindings::futhark_shape_{rust_type}_{dim}d(ctx, ptr as *mut bindings::futhark_{rust_type}_{dim}d)
    }}
    unsafe fn values(ctx: &crate::context::FutharkContext, ptr: *mut Self, dst: *mut Self::RustType)
    {{
        let ctx = ctx.ptr();
        bindings::futhark_values_{rust_type}_{dim}d(ctx, ptr, dst);
        // Sync the values to the array.
        bindings::futhark_context_sync(ctx);
    }}
    unsafe fn free(ctx: &crate::context::FutharkContext, ptr: *mut Self)
    {{
        let ctx = ctx.ptr();
        bindings::futhark_free_{rust_type}_{dim}d(ctx, ptr);
    }}}}"#, rust_type=captures[1].to_owned(), dim=dim, array_dim=array_dim_expansion(dim)
    ).expect("Write failed!");
    buffer
}

pub(crate) fn gen_impl_futhark_types(input: &Vec<String>) -> String {
    let mut buffer = String::new();
    let mut buffer2 = String::new();
    writeln!(&mut buffer, "use crate::bindings::*;").expect("Write failed!");
    for t in input {
        println!("{}", t);
        writeln!(&mut buffer, "{}", gen_impl_futhark_type(&t)).expect("Write failed!");
        writeln!(&mut buffer2, "{}", gen_specific_type(&t)).expect("Write failed!");
    }
    writeln!(&mut buffer, "{}", buffer2).expect("Write failed!");
    buffer
}
