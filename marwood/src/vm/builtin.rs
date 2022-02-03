use crate::number::Number;
use crate::vm::vcell::VCell;
use crate::vm::vector::Vector;
use crate::vm::Error::{
    ExpectedPairButFound, InvalidArgs, InvalidNumArgs, InvalidSyntax, InvalidVectorIndex,
};
use crate::vm::{Error, Vm};
use std::rc::Rc;

/// Built Ins
///
/// Built ins are implementations for scheme primitive or library procedures
/// that either must be written in rust (e.g. type predicates), or are implemented
/// in rust for performance reasons.
///
/// Like normal procedures, these procdues are the target of a CALL or TCALL
/// instruction where the procedure being applied is a VCell::BuiltIn object
/// instead of a Lambda or Closure.
///
/// The CALL and TCALL instruction will bypass normal frame setup and pass
/// control to the procedure in this file.
///
/// Because a call frame was not setup, the top of the stack for a built in
/// is argc, and immediately preceding argc are the arguments being applied.
/// All builtins must pop the argc value along with argc* arguments off
/// the stack before returning.

impl Vm {
    pub fn load_builtins(&mut self) {
        self.load_builtin("*", multiply);
        self.load_builtin("/", divide);
        self.load_builtin("+", plus);
        self.load_builtin("-", minus);
        self.load_builtin("<", lt);
        self.load_builtin("<=", lteq);
        self.load_builtin("=", num_equal);
        self.load_builtin(">", gt);
        self.load_builtin(">=", gteq);
        self.load_builtin("%", remainder);
        self.load_builtin("abs", abs);
        self.load_builtin("append", append);
        self.load_builtin("boolean?", is_boolean);
        self.load_builtin("car", car);
        self.load_builtin("cdr", cdr);
        self.load_builtin("char?", is_char);
        self.load_builtin("cons", cons);
        self.load_builtin("eq?", eq);
        self.load_builtin("equal?", equal);
        self.load_builtin("eqv?", eqv);
        self.load_builtin("even?", even);
        self.load_builtin("exact->inexact", exact_inexact);
        self.load_builtin("inexact->exact", inexact_exact);
        self.load_builtin("make-vector", make_vector);
        self.load_builtin("list?", is_list);
        self.load_builtin("list-ref", list_ref);
        self.load_builtin("list-tail", list_tail);
        self.load_builtin("negative?", negative);
        self.load_builtin("not", not);
        self.load_builtin("null?", is_null);
        self.load_builtin("number?", is_number);
        self.load_builtin("odd?", odd);
        self.load_builtin("pair?", is_pair);
        self.load_builtin("port?", is_port);
        self.load_builtin("positive?", positive);
        self.load_builtin("procedure?", is_procedure);
        self.load_builtin("quotient", quotient);
        self.load_builtin("remainder", remainder);
        self.load_builtin("reverse", reverse);
        self.load_builtin("set-car!", set_car);
        self.load_builtin("set-cdr!", set_cdr);
        self.load_builtin("string?", is_string);
        self.load_builtin("symbol?", is_symbol);
        self.load_builtin("vector", vector);
        self.load_builtin("vector-length", vector_length);
        self.load_builtin("vector->list", vector_to_list);
        self.load_builtin("list->vector", list_to_vector);
        self.load_builtin("vector-ref", vector_ref);
        self.load_builtin("vector-set!", vector_set);
        self.load_builtin("vector-fill!", vector_fill);
        self.load_builtin("vector?", is_vector);
        self.load_builtin("zero?", zero);
    }

    fn load_builtin(&mut self, symbol: &str, func: fn(&mut Vm) -> Result<VCell, Error>) {
        let syscall = self.heap.put(VCell::syscall(func));
        let symbol = self.heap.put(VCell::symbol(symbol));
        let slot = self.globenv.get_binding(symbol.as_ptr().unwrap());
        self.globenv.put_slot(slot, syscall);
    }
}

/// Pop Argc
///
/// Pop the number of arguments applied to a procedure off the top of
/// the stack. Return an error if they don't match the expected.
fn pop_argc(vm: &mut Vm, min: usize, max: Option<usize>, proc: &str) -> Result<usize, Error> {
    let argc = vm.stack.pop()?.as_argc()?;
    if argc < min || (max.is_some() && argc > max.unwrap()) {
        Err(InvalidNumArgs(proc.into()))
    } else {
        Ok(argc)
    }
}

///
/// List Procedures
///
fn car(vm: &mut Vm) -> Result<VCell, Error> {
    pop_argc(vm, 1, Some(1), "car")?;
    Ok(match vm.heap.get(vm.stack.pop()?) {
        VCell::Pair(car, _) => VCell::ptr(car),
        arg => return Err(ExpectedPairButFound(vm.heap.get_as_cell(&arg).to_string())),
    })
}

fn cdr(vm: &mut Vm) -> Result<VCell, Error> {
    pop_argc(vm, 1, Some(1), "cdr")?;
    Ok(match vm.heap.get(vm.stack.pop()?) {
        VCell::Pair(_, cdr) => VCell::ptr(cdr),
        arg => return Err(ExpectedPairButFound(vm.heap.get_as_cell(&arg).to_string())),
    })
}

fn cons(vm: &mut Vm) -> Result<VCell, Error> {
    pop_argc(vm, 2, Some(2), "cons")?;
    let cdr = vm.stack.pop()?.as_ptr()?;
    let car = vm.stack.pop()?.as_ptr()?;
    Ok(VCell::Pair(car, cdr))
}

fn set_car(vm: &mut Vm) -> Result<VCell, Error> {
    pop_argc(vm, 2, Some(2), "set-car!")?;
    let obj = vm.stack.pop()?.clone();
    let pair = vm.stack.pop()?.clone();
    let new_pair = match vm.heap.get(&pair) {
        VCell::Pair(_, cdr) => VCell::Pair(obj.as_ptr()?, cdr),
        _ => {
            return Err(InvalidSyntax("set-car! expected a pair".into()));
        }
    };
    *vm.heap.get_at_index_mut(pair.as_ptr()?) = new_pair;
    Ok(VCell::Void)
}

fn set_cdr(vm: &mut Vm) -> Result<VCell, Error> {
    pop_argc(vm, 2, Some(2), "set-cdr!")?;
    let obj = vm.stack.pop()?.clone();
    let pair = vm.stack.pop()?.clone();
    let new_pair = match vm.heap.get(&pair) {
        VCell::Pair(car, _) => VCell::Pair(car, obj.as_ptr()?),
        _ => {
            return Err(InvalidSyntax("set-cdr! expected a pair".into()));
        }
    };
    *vm.heap.get_at_index_mut(pair.as_ptr()?) = new_pair;
    Ok(VCell::Void)
}

/// Clone List
///
/// This function clones list, returning an error if list is
/// not a list or is improper.
///
/// # Arguments
/// `vm` - The vm in which to allocate the list
/// `list` - The list to clone
fn clone_list(vm: &mut Vm, list: VCell) -> Result<(VCell, VCell), Error> {
    let mut rest = list.clone();
    if !rest.is_pair() {
        return Err(ExpectedPairButFound(vm.heap.get_as_cell(&rest).to_string()));
    }

    let mut head = VCell::Nil;
    let mut tail = VCell::Nil;

    let nil = vm.heap.put(VCell::Nil).as_ptr()?;

    loop {
        let pair = vm.heap.put(VCell::Pair(rest.as_car()?.as_ptr()?, nil));
        if head.is_nil() {
            head = pair.clone();
        }
        if tail.is_nil() {
            tail = pair.clone();
        } else {
            let last_pair = vm.heap.get(&tail);
            *vm.heap.get_at_index_mut(tail.as_ptr()?) =
                VCell::Pair(last_pair.as_car()?.as_ptr()?, pair.as_ptr()?);
            tail = pair;
        }
        rest = vm.heap.get(&rest.as_cdr()?);
        if !rest.is_pair() {
            if rest.is_nil() {
                return Ok((head, tail));
            } else {
                return Err(InvalidSyntax(format!(
                    "{} is an improper list",
                    vm.heap.get_as_cell(&list)
                )));
            }
        }
    }
}

fn append(vm: &mut Vm) -> Result<VCell, Error> {
    let argc = pop_argc(vm, 0, None, "append")?;
    if argc == 0 {
        return Ok(VCell::Nil);
    }
    let mut tail = vm.stack.pop()?.clone();
    tail = match vm.heap.get(&tail) {
        VCell::Nil | VCell::Pair(_, _) => tail,
        _ => {
            let nil = vm.heap.put(VCell::Nil).as_ptr()?;
            vm.heap.put(VCell::Pair(tail.as_ptr()?, nil))
        }
    };

    for _ in 0..(argc - 1) {
        let list = vm.heap.get(&vm.stack.pop()?.clone());
        match list {
            VCell::Nil => {
                continue;
            }
            VCell::Pair(_, _) => {}
            vcell => {
                return Err(ExpectedPairButFound(
                    vm.heap.get_as_cell(&vcell).to_string(),
                ));
            }
        }
        let (head, sub_tail) = clone_list(vm, list)?;
        let sub_pair = vm.heap.get(&sub_tail);
        *vm.heap.get_at_index_mut(sub_tail.as_ptr()?) =
            VCell::Pair(sub_pair.as_car()?.as_ptr()?, tail.as_ptr()?);
        tail = head;
    }

    Ok(tail)
}

fn reverse(vm: &mut Vm) -> Result<VCell, Error> {
    let _ = pop_argc(vm, 1, Some(1), "reverse")?;
    let list = vm.heap.get(vm.stack.pop()?);
    let mut rest = list.clone();
    if !rest.is_pair() {
        return if rest.is_nil() {
            Ok(rest)
        } else {
            Err(ExpectedPairButFound(vm.heap.get_as_cell(&rest).to_string()))
        };
    }
    let mut tail = vm.heap.put(VCell::Nil);
    loop {
        tail = vm
            .heap
            .put(VCell::Pair(rest.as_car()?.as_ptr()?, tail.as_ptr()?));
        rest = vm.heap.get(&rest.as_cdr()?);
        if !rest.is_pair() {
            if rest.is_nil() {
                break;
            } else {
                return Err(InvalidSyntax(format!(
                    "{} is an improper list",
                    vm.heap.get_as_cell(&list)
                )));
            }
        }
    }
    Ok(tail)
}

/// Get List Tail
///
/// Return the list tail at the given index, or an error if the
/// index is invalid.
fn get_list_tail(vm: &mut Vm, list: &VCell, idx: usize) -> Result<VCell, Error> {
    if !list.is_pair() && !list.is_nil() {
        return Err(ExpectedPairButFound(vm.heap.get_as_cell(list).to_string()));
    }

    let mut rest = list.clone();
    let mut rest_idx = idx;
    loop {
        if rest_idx == 0 {
            return Ok(rest);
        }
        rest_idx -= 1;
        rest = vm.heap.get(&rest.as_cdr()?);
        if !rest.is_pair() && rest_idx != 0 {
            return Err(InvalidSyntax(format!(
                "{} is out of range for {}",
                idx,
                vm.heap.get_as_cell(list)
            )));
        }
    }
}

fn list_ref(vm: &mut Vm) -> Result<VCell, Error> {
    let _ = pop_argc(vm, 2, Some(2), "list-tail")?;
    let idx = pop_index(vm)?;
    let list = vm.heap.get(vm.stack.pop()?);
    if !list.is_pair() && !list.is_nil() {
        return Err(ExpectedPairButFound(vm.heap.get_as_cell(&list).to_string()));
    }
    let tail = get_list_tail(vm, &list, idx)?;
    match tail {
        VCell::Pair(car, _) => Ok(VCell::Ptr(car)),
        _ => Err(InvalidSyntax(format!(
            "{} is out of range for {}",
            idx,
            vm.heap.get_as_cell(&list)
        ))),
    }
}

fn list_tail(vm: &mut Vm) -> Result<VCell, Error> {
    let _ = pop_argc(vm, 2, Some(2), "list-tail")?;
    let idx = pop_index(vm)?;
    let list = vm.heap.get(vm.stack.pop()?);
    if !list.is_pair() && !list.is_nil() {
        return Err(ExpectedPairButFound(vm.heap.get_as_cell(&list).to_string()));
    }
    get_list_tail(vm, &list, idx)
}

///
/// Predicates
///
fn is_boolean(vm: &mut Vm) -> Result<VCell, Error> {
    pop_argc(vm, 1, Some(1), "boolean?")?;
    let result = vm.heap.get(vm.stack.pop()?);
    Ok(result.is_boolean().into())
}

fn is_char(vm: &mut Vm) -> Result<VCell, Error> {
    pop_argc(vm, 1, Some(1), "char?")?;
    let _ = vm.stack.pop()?;
    Ok(false.into())
}

fn is_list(vm: &mut Vm) -> Result<VCell, Error> {
    pop_argc(vm, 1, Some(1), "list?")?;
    let mut rest = vm.heap.get(vm.stack.pop()?);
    loop {
        if !rest.is_pair() {
            return Ok(rest.is_nil().into());
        }
        rest = vm.heap.get(&rest.as_cdr()?);
    }
}

fn is_null(vm: &mut Vm) -> Result<VCell, Error> {
    pop_argc(vm, 1, Some(1), "null?")?;
    let result = vm.heap.get(vm.stack.pop()?);
    Ok(result.is_nil().into())
}

fn is_number(vm: &mut Vm) -> Result<VCell, Error> {
    pop_argc(vm, 1, Some(1), "number?")?;
    let result = vm.heap.get(vm.stack.pop()?);
    Ok(result.is_number().into())
}

fn is_pair(vm: &mut Vm) -> Result<VCell, Error> {
    pop_argc(vm, 1, Some(1), "pair?")?;
    let result = vm.heap.get(vm.stack.pop()?);
    Ok(result.is_pair().into())
}

fn is_port(vm: &mut Vm) -> Result<VCell, Error> {
    pop_argc(vm, 1, Some(1), "port?")?;
    let _ = vm.stack.pop()?;
    Ok(false.into())
}

fn is_procedure(vm: &mut Vm) -> Result<VCell, Error> {
    pop_argc(vm, 1, Some(1), "procedure?")?;
    let result = vm.heap.get(vm.stack.pop()?);
    Ok(result.is_procedure().into())
}

fn is_string(vm: &mut Vm) -> Result<VCell, Error> {
    pop_argc(vm, 1, Some(1), "string?")?;
    let _ = vm.stack.pop()?;
    Ok(false.into())
}

fn is_symbol(vm: &mut Vm) -> Result<VCell, Error> {
    pop_argc(vm, 1, Some(1), "symbol?")?;
    let result = vm.heap.get(vm.stack.pop()?);
    Ok(result.is_symbol().into())
}

fn is_vector(vm: &mut Vm) -> Result<VCell, Error> {
    pop_argc(vm, 1, Some(1), "vector?")?;
    let result = vm.heap.get(vm.stack.pop()?);
    Ok(result.is_vector().into())
}

fn eq(vm: &mut Vm) -> Result<VCell, Error> {
    pop_argc(vm, 2, Some(2), "eq?")?;
    let left = vm.stack.pop()?.clone();
    let right = vm.stack.pop()?.clone();
    Ok(vm.eqv(&left, &right)?.into())
}

fn eqv(vm: &mut Vm) -> Result<VCell, Error> {
    pop_argc(vm, 2, Some(2), "eqv?")?;
    let left = vm.stack.pop()?.clone();
    let right = vm.stack.pop()?.clone();
    Ok(vm.eqv(&left, &right)?.into())
}

fn equal(vm: &mut Vm) -> Result<VCell, Error> {
    pop_argc(vm, 2, Some(2), "eqv?")?;
    let left = vm.stack.pop()?.clone();
    let right = vm.stack.pop()?.clone();
    Ok(vm.equal(&left, &right)?.into())
}

fn not(vm: &mut Vm) -> Result<VCell, Error> {
    pop_argc(vm, 1, Some(1), "not")?;
    Ok(match vm.heap.get(vm.stack.pop()?) {
        VCell::Bool(val) => !val,
        _ => false,
    }
    .into())
}

///
/// Numerical Procedures
///
///

fn pop_number(vm: &mut Vm) -> Result<Number, Error> {
    match vm.heap.get(vm.stack.pop()?) {
        VCell::Number(num) => Ok(num),
        vcell => {
            return Err(InvalidSyntax(format!(
                "{} is not a valid number",
                vm.heap.get_as_cell(&vcell)
            )))
        }
    }
}

fn pop_integer(vm: &mut Vm) -> Result<Number, Error> {
    match pop_number(vm) {
        Ok(num) if num.is_integer() => Ok(num),
        Ok(num) => return Err(InvalidSyntax(format!("{} is not a valid integer", num))),
        Err(e) => Err(e),
    }
}

fn num_equal(vm: &mut Vm) -> Result<VCell, Error> {
    num_comp(vm, "=", |x, y| x == y)
}

fn lt(vm: &mut Vm) -> Result<VCell, Error> {
    num_comp(vm, "<", |x, y| x < y)
}

fn gt(vm: &mut Vm) -> Result<VCell, Error> {
    num_comp(vm, ">", |x, y| x > y)
}

fn lteq(vm: &mut Vm) -> Result<VCell, Error> {
    num_comp(vm, "<=", |x, y| x <= y)
}

fn gteq(vm: &mut Vm) -> Result<VCell, Error> {
    num_comp(vm, ">=", |x, y| x >= y)
}

fn num_comp(
    vm: &mut Vm,
    name: &str,
    comp: impl Fn(&Number, &Number) -> bool,
) -> Result<VCell, Error> {
    let argc = pop_argc(vm, 1, None, name)?;
    let mut result = true;

    let mut y = match vm.heap.get(vm.stack.pop()?) {
        VCell::Number(val) => val,
        _ => {
            result = false;
            Number::from(0)
        }
    };

    for _ in 0..argc - 1 {
        match vm.heap.get(vm.stack.pop()?) {
            VCell::Number(x) if comp(&x, &y) => {
                y = x;
                continue;
            }
            _ => {
                result = false;
            }
        };
    }

    Ok(result.into())
}

fn zero(vm: &mut Vm) -> Result<VCell, Error> {
    num_unary_predicate(vm, "zero?", |x| x == &Number::from(0))
}

fn positive(vm: &mut Vm) -> Result<VCell, Error> {
    num_unary_predicate(vm, "positive?", |x| x > &Number::from(0))
}

fn negative(vm: &mut Vm) -> Result<VCell, Error> {
    num_unary_predicate(vm, "negative?", |x| x < &Number::from(0))
}

fn odd(vm: &mut Vm) -> Result<VCell, Error> {
    num_unary_predicate(vm, "odd?", |x| {
        x % &Number::from(2) != Some(Number::from(0))
    })
}

fn even(vm: &mut Vm) -> Result<VCell, Error> {
    num_unary_predicate(vm, "even?", |x| {
        x % &Number::from(2) == Some(Number::from(0))
    })
}

fn num_unary_predicate(
    vm: &mut Vm,
    name: &str,
    predicate: impl Fn(&Number) -> bool,
) -> Result<VCell, Error> {
    pop_argc(vm, 1, Some(1), name)?;
    let x = match vm.heap.get(vm.stack.pop()?) {
        VCell::Number(val) => val,
        _ => return Ok(false.into()),
    };
    Ok(predicate(&x).into())
}

fn plus(vm: &mut Vm) -> Result<VCell, Error> {
    let argc = pop_argc(vm, 0, None, "+")?;
    let mut sum = Number::from(0);
    for _ in 0..argc {
        sum += match vm.heap.get(vm.stack.pop()?) {
            VCell::Number(n) => n,
            vcell => {
                return Err(InvalidArgs(
                    "+".to_string(),
                    "number".to_string(),
                    vm.heap.get_as_cell(&vcell).to_string(),
                ));
            }
        }
    }
    Ok(sum.into())
}

fn minus(vm: &mut Vm) -> Result<VCell, Error> {
    let argc = pop_argc(vm, 1, None, "-")?;
    let mut result = Number::from(0);
    for _ in 0..(argc - 1) {
        result += match vm.heap.get(vm.stack.pop()?) {
            VCell::Number(n) => n,
            vcell => {
                return Err(InvalidArgs(
                    "-".to_string(),
                    "number".to_string(),
                    vm.heap.get_as_cell(&vcell).to_string(),
                ));
            }
        }
    }

    if let VCell::Number(n) = vm.heap.get(vm.stack.pop()?) {
        result = n - result;
    }

    if argc == 1 {
        result *= Number::from(-1);
    }

    Ok(VCell::Number(result))
}

fn multiply(vm: &mut Vm) -> Result<VCell, Error> {
    let argc = pop_argc(vm, 0, None, "*")?;
    let mut result = Number::from(1);
    for _ in 0..argc {
        result *= match vm.heap.get(vm.stack.pop()?) {
            VCell::Number(n) => n,
            vcell => {
                return Err(InvalidArgs(
                    "*".to_string(),
                    "number".to_string(),
                    vm.heap.get_as_cell(&vcell).to_string(),
                ));
            }
        }
    }
    Ok(VCell::Number(result))
}

fn divide(vm: &mut Vm) -> Result<VCell, Error> {
    let argc = pop_argc(vm, 1, Some(2), "/")?;
    let y = pop_number(vm)?;

    if y.is_zero() {
        return Err(InvalidSyntax("/ is undefined for 0".into()));
    }

    if argc == 1 {
        let result = Number::from(1) / y;
        Ok(result.into())
    } else {
        let x = pop_number(vm)?;
        let result = x / y;
        Ok(result.into())
    }
}

fn remainder(vm: &mut Vm) -> Result<VCell, Error> {
    pop_argc(vm, 2, Some(2), "remainder")?;
    let y = pop_integer(vm)?;
    let x = pop_integer(vm)?;

    if y.is_zero() {
        return Err(InvalidSyntax("remainder is undefined for 0".into()));
    }

    let result = match &x % &y {
        Some(num) => num,
        None => {
            return Err(InvalidSyntax(format!(
                "remainder is undefined for {} % {}",
                x, y
            )))
        }
    };
    Ok(result.into())
}

fn quotient(vm: &mut Vm) -> Result<VCell, Error> {
    pop_argc(vm, 2, Some(2), "quotient")?;
    let y = pop_integer(vm)?;
    let x = pop_integer(vm)?;

    if y.is_zero() {
        return Err(InvalidSyntax("quotient is undefined for 0".into()));
    }

    let result = match x.quotient(&y) {
        Some(num) => num,
        None => {
            return Err(InvalidSyntax(format!(
                "quotient is undefined for {} % {}",
                x, y
            )))
        }
    };
    Ok(result.into())
}

fn exact_inexact(vm: &mut Vm) -> Result<VCell, Error> {
    pop_argc(vm, 1, Some(1), "exact->inexact")?;
    let x = pop_number(vm)?;
    match x.to_inexact() {
        Some(num) => Ok(num.into()),
        None => Ok(x.into()),
    }
}

fn inexact_exact(vm: &mut Vm) -> Result<VCell, Error> {
    pop_argc(vm, 1, Some(1), "inexact->exact")?;
    let x = pop_number(vm)?;
    match x.to_exact() {
        Some(num) => Ok(num.into()),
        None => Ok(x.into()),
    }
}

fn abs(vm: &mut Vm) -> Result<VCell, Error> {
    pop_argc(vm, 1, Some(1), "exact->inexact")?;
    let x = pop_number(vm)?;
    let x = x.abs();
    Ok(x.into())
}

///
/// Vector Builtins
///

fn pop_vector(vm: &mut Vm) -> Result<Rc<Vector>, Error> {
    match vm.heap.get(vm.stack.pop()?) {
        VCell::Vector(vector) => Ok(vector),
        vcell => {
            return Err(InvalidSyntax(format!(
                "{} is not a vector",
                vm.heap.get_as_cell(&vcell)
            )))
        }
    }
}

fn pop_index(vm: &mut Vm) -> Result<usize, Error> {
    match vm.heap.get(vm.stack.pop()?) {
        VCell::Number(num) => num
            .to_usize()
            .ok_or_else(|| InvalidSyntax(format!("{} is not a valid index", num))),
        vcell => {
            return Err(InvalidSyntax(format!(
                "{} is not a valid index",
                vm.heap.get_as_cell(&vcell)
            )))
        }
    }
}

fn vector(vm: &mut Vm) -> Result<VCell, Error> {
    let len = pop_argc(vm, 0, None, "vector")?;
    let mut outv = vec![VCell::Undefined; len];
    for idx in (0..len).rev() {
        *outv.get_mut(idx).unwrap() = vm.stack.pop()?.clone();
    }
    Ok(VCell::vector(outv))
}

fn make_vector(vm: &mut Vm) -> Result<VCell, Error> {
    let err = || InvalidSyntax("make-vector requires an integer size".into());
    let argc = pop_argc(vm, 1, Some(2), "make-vector")?;
    let fill = match argc {
        2 => vm.stack.pop()?.clone(),
        _ => vm.heap.put(VCell::Number(Number::from(0))),
    };

    let len = match vm.heap.get(vm.stack.pop()?) {
        VCell::Number(num) => num.to_usize().ok_or_else(err)?,
        _ => return Err(err()),
    };

    let outv = vec![fill; len];
    Ok(VCell::vector(outv))
}

fn vector_length(vm: &mut Vm) -> Result<VCell, Error> {
    pop_argc(vm, 1, Some(1), "vector-length")?;
    let vector = pop_vector(vm)?;
    Ok(Number::from(vector.len() as i64).into())
}

fn vector_ref(vm: &mut Vm) -> Result<VCell, Error> {
    pop_argc(vm, 2, Some(2), "vector-ref")?;
    let idx = pop_index(vm)?;
    let vector = pop_vector(vm)?;
    match vector.get(idx) {
        Some(value) => Ok(value),
        None => Err(InvalidVectorIndex(idx, vector.len())),
    }
}

fn vector_set(vm: &mut Vm) -> Result<VCell, Error> {
    pop_argc(vm, 3, Some(3), "vector-set!")?;
    let value = vm.stack.pop()?.clone();
    let idx = pop_index(vm)?;
    let vector = pop_vector(vm)?;
    if idx > vector.len() - 1 {
        return Err(InvalidVectorIndex(idx, vector.len()));
    }
    vector.put(idx, value);
    Ok(VCell::Void)
}

fn vector_fill(vm: &mut Vm) -> Result<VCell, Error> {
    pop_argc(vm, 2, Some(2), "vector-fill!")?;
    let value = vm.heap.get(vm.stack.pop()?);
    let vector = pop_vector(vm)?;
    for idx in 0..vector.len() {
        vector.put(idx, value.clone());
    }
    Ok(VCell::Void)
}

fn vector_to_list(vm: &mut Vm) -> Result<VCell, Error> {
    pop_argc(vm, 1, Some(1), "vector->list")?;
    let vector = pop_vector(vm)?;
    let mut tail = vm.heap.put(VCell::Nil);
    for idx in (0..vector.len()).rev() {
        let car = vector.get(idx).unwrap();
        tail = vm.heap.put(VCell::Pair(car.as_ptr()?, tail.as_ptr()?));
    }
    Ok(tail)
}

fn list_to_vector(vm: &mut Vm) -> Result<VCell, Error> {
    pop_argc(vm, 1, Some(1), "list->vector")?;
    let mut list = vm.heap.get(vm.stack.pop()?);
    if !list.is_pair() {
        if list.is_nil() {
            return Ok(VCell::vector(vec![]));
        } else {
            return Err(ExpectedPairButFound(vm.heap.get_as_cell(&list).to_string()));
        }
    }
    let mut outv = vec![];
    while list.is_pair() {
        outv.push(list.as_car()?);
        list = vm.heap.get(&list.as_cdr()?);
    }
    Ok(VCell::vector(outv))
}
