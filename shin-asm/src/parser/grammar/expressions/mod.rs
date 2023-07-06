mod atom;

use super::*;

pub(super) const EXPR_FIRST: TokenSet = LHS_FIRST;

pub(super) fn expr(p: &mut Parser<'_>) -> Option<CompletedMarker> {
    expr_bp(p, None, 1)
}

enum Associativity {
    Left,
    // even though we do not have any right-associative operators right now, I still want to keep this code to possibly introduce them in the future
    #[allow(dead_code)]
    Right,
}

/// Binding powers of operators for a Pratt parser.
///
/// See <https://matklad.github.io/2020/04/13/simple-but-powerful-pratt-parsing.html>
///
/// Note that Rust doesn't define associativity for some infix operators (e.g. `==` and `..`) and
/// requires parentheses to disambiguate. We just treat them as left associative.
#[rustfmt::skip]
fn current_op(p: &Parser<'_>) -> (u8, SyntaxKind, Associativity) {
    use Associativity::*;
    const NOT_AN_OP: (u8, SyntaxKind, Associativity) = (0, T![@], Left);
    match p.current() {
        T![||]   => (3,  T![||],  Left),
        T![|]    => (6,  T![|],   Left),
        T![>>]   => (9,  T![>>],  Left),
        T![>=]   => (5,  T![>=],  Left),
        T![>]    => (5,  T![>],   Left),
        T![==]   => (5,  T![==],  Left),
        T![<=]   => (5,  T![<=],  Left),
        T![<<]   => (9,  T![<<],  Left),
        T![<]    => (5,  T![<],   Left),
        T![+]    => (10, T![+],   Left),
        T![^]    => (7,  T![^],   Left),
        T![mod]  => (11, T![mod], Left),
        T![&&]   => (4,  T![&&],  Left),
        T![&]    => (8,  T![&],   Left),
        T![/]    => (11, T![/],   Left),
        T![*]    => (11, T![*],   Left),
        T![./]   => (11, T![./],  Left),
        T![.*]   => (11, T![.*],  Left),
        T![!=]   => (5,  T![!=],  Left),
        T![-]    => (10, T![-],   Left),
        _                      => NOT_AN_OP
    }
}

// Parses expression with binding power of at least bp.
fn expr_bp(p: &mut Parser<'_>, m: Option<Marker>, bp: u8) -> Option<CompletedMarker> {
    let m = m.unwrap_or_else(|| {
        let m = p.start();
        m
    });

    if !p.at_ts(EXPR_FIRST) {
        p.err_and_bump_unmatching("expected expression", atom::EXPR_RECOVERY_SET);
        m.abandon(p);
        return None;
    }
    let mut lhs = match lhs(p) {
        Some(lhs) => lhs.extend_to(p, m),
        None => {
            m.abandon(p);
            return None;
        }
    };

    loop {
        let (op_bp, op, associativity) = current_op(p);
        if op_bp < bp {
            break;
        }
        let m = lhs.precede(p);
        p.bump(op);

        let op_bp = match associativity {
            Associativity::Left => op_bp + 1,
            Associativity::Right => op_bp,
        };
        expr_bp(p, None, op_bp);
        lhs = m.complete(p, BIN_EXPR);
    }
    Some(lhs)
}

const LHS_FIRST: TokenSet = atom::ATOM_EXPR_FIRST.union(TokenSet::new(&[T![!], T![~], T![-]]));

fn lhs(p: &mut Parser<'_>) -> Option<CompletedMarker> {
    let m;
    let kind = match p.current() {
        T![~] | T![!] | T![-] => {
            m = p.start();
            p.bump_any();
            PREFIX_EXPR
        }
        _ => {
            // test expression_after_block
            // fn foo() {
            //    let mut p = F{x: 5};
            //    {p}.x = 10;
            // }
            let lhs = atom::atom_expr(p)?;
            return Some(postfix_expr(p, lhs));
        }
    };
    // parse the interior of the unary expression
    expr_bp(p, None, 255);
    Some(m.complete(p, kind))
}

fn postfix_expr(p: &mut Parser<'_>, mut lhs: CompletedMarker) -> CompletedMarker {
    loop {
        lhs = match p.current() {
            T!['('] => call_expr(p, lhs),
            _ => break,
        };
    }
    lhs
}

fn call_expr(p: &mut Parser<'_>, lhs: CompletedMarker) -> CompletedMarker {
    assert!(p.at(T!['(']));
    let m = lhs.precede(p);
    call_arg_list(p);
    m.complete(p, CALL_EXPR)
}

fn call_arg_list(p: &mut Parser<'_>) {
    assert!(p.at(T!['(']));
    let m = p.start();
    p.bump(T!['(']);
    delimited(
        p,
        TokenSet::new(&[T![')']]),
        T![,],
        EXPR_FIRST,
        |p: &mut Parser<'_>| expr(p).is_some(),
    );
    p.expect(T![')']);
    m.complete(p, CALL_EXPR_ARG_LIST);
}
