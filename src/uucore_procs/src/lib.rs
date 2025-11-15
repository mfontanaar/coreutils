// This file is part of the uutils coreutils package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.
//
// spell-checker:ignore SIGSEGV

//! A collection of procedural macros for uutils.
#![deny(missing_docs)]

// For testing no macro code we use proc_macro2
use proc_macro2::TokenStream as TokenStream2;
use proc_macro2::TokenTree as TokenTree2;

use proc_macro::TokenStream;
use quote::quote;

fn parse_macro_args(attr: TokenStream2) -> Result<bool, &'static str> {
    let mut iter = attr.into_iter();

    // No attribute → false
    let Some(first) = iter.next() else {
        return Ok(false);
    };

    // First must be identifier: `ignore_sigpipe`
    let name = match first {
        TokenTree2::Ident(id) => id.to_string(),
        _ => return Err("expected `ignore_sigpipe = <bool>` or nothing"),
    };
    if name != "ignore_sigpipe" {
        return Err("expected `ignore_sigpipe`");
    }

    // Expect "="
    match iter.next() {
        Some(TokenTree2::Punct(p)) if p.as_char() == '=' => {}
        _ => return Err("expected `=`"),
    }

    // Expect boolean value
    let value = match iter.next() {
        Some(TokenTree2::Ident(id)) => match id.to_string().as_str() {
            "true" => true,
            "false" => false,
            _ => return Err("expected `true` or `false`"),
        },
        _ => return Err("expected boolean literal"),
    };

    // No trailing garbage
    if iter.next().is_some() {
        return Err("unexpected extra tokens");
    }

    Ok(value)
}

//## rust proc-macro background info
//* ref: <https://dev.to/naufraghi/procedural-macro-in-rust-101-k3f> @@ <http://archive.is/Vbr5e>
//* ref: [path construction from LitStr](https://oschwald.github.io/maxminddb-rust/syn/struct.LitStr.html) @@ <http://archive.is/8YDua>

/// A procedural macro to define the main function of a uutils binary.
/// Unless specified otherwise, it will treat SIGPIPE as SIG_DFL,
/// which is POSIX-compliant behavior for most utilities.
/// Usage: #[main(ignore_sigpipe = true|false)] (default: false)
#[proc_macro_attribute]
pub fn main(args: TokenStream, stream: TokenStream) -> TokenStream {
    let ignore_sigpipe = match parse_macro_args(args.into()) {
        Ok(v) => v,
        Err(msg) => return quote! { compile_error!(#msg); }.into(),
    };

    let stream = proc_macro2::TokenStream::from(stream);

    let new = quote!(
        pub fn uumain(args: impl uucore::Args) -> i32 {
            #stream

            // disable rust signal handlers (otherwise processes don't dump core after e.g. one SIGSEGV)
            #[cfg(unix)]
            uucore::disable_rust_signal_handlers().expect("Disabling rust signal handlers failed");

            // map SIGPIPE to SIG_DFL unless it should be ignored
            const __IGNORE_SIGPIPE: bool = #ignore_sigpipe;
            if !__IGNORE_SIGPIPE {
                #[cfg(unix)]
                uucore::sigpipe::fix_sigpipe_handling();
            }

            let result = uumain(args);
            match result {
                Ok(()) => uucore::error::get_exit_code(),
                Err(e) => {
                    let s = format!("{e}");
                    if s != "" {
                        uucore::show_error!("{s}");
                    }
                    if e.usage() {
                        eprintln!("Try '{} --help' for more information.", uucore::execution_phrase());
                    }
                    e.code()
                }
            }
        }
    );

    TokenStream::from(new)
}

#[cfg(test)]
mod tests {
    use super::parse_macro_args;
    use proc_macro2::TokenStream;
    use quote::quote;

    #[test]
    fn test_default() {
        // No argument → default false
        let args: TokenStream = quote! {}.into();
        let flag = parse_macro_args(args.into()).unwrap();
        assert_eq!(flag, false);
    }

    #[test]
    fn test_flag_true() {
        let args: TokenStream = quote! { ignore_sigpipe = true }.into();
        let flag = parse_macro_args(args.into()).unwrap();
        assert_eq!(flag, true);
    }

    #[test]
    fn test_flag_false() {
        let args: TokenStream = quote! { ignore_sigpipe = false }.into();
        let flag = parse_macro_args(args.into()).unwrap();
        assert_eq!(flag, false);
    }

    #[test]
    fn test_invalid_keyword() {
        let args: TokenStream = quote! { foo = true }.into();
        let err = parse_macro_args(args.into()).unwrap_err();
        assert_eq!(err, "expected `ignore_sigpipe`");
    }

    #[test]
    fn test_invalid_value() {
        let args: TokenStream = quote! { ignore_sigpipe = maybe }.into();
        let err = parse_macro_args(args.into()).unwrap_err();
        assert_eq!(err, "expected `true` or `false`");
    }

    #[test]
    fn test_extra_tokens() {
        let args: TokenStream = quote! { ignore_sigpipe = true extra }.into();
        let err = parse_macro_args(args.into()).unwrap_err();
        assert_eq!(err, "unexpected extra tokens");
    }

    #[test]
    fn test_quoted_string() {
        let args: TokenStream = quote! { ignore_sigpipe = "true" }.into();
        let err = parse_macro_args(args.into()).unwrap_err();
        assert_eq!(err, "expected boolean literal");
    }
}
