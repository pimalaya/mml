//! Module dedicated to email address utils.

use std::{borrow::Cow, collections::HashSet};

use mail_builder::headers::address as builder;
use mail_parser as parser;
use once_cell::sync::Lazy;
use regex::Regex;

/// Regex used to detect if an email address is a noreply one.
///
/// Matches usual names like `no_reply`, `noreply`, but also
/// `do-not.reply`.
static NO_REPLY: Lazy<Regex> = Lazy::new(|| Regex::new("(?i:not?[_\\-\\.]?reply)").unwrap());

pub(crate) fn is_empty(header: &parser::HeaderValue) -> bool {
    match header {
        parser::HeaderValue::Address(parser::Address::List(addrs)) => addrs.is_empty(),
        parser::HeaderValue::Address(parser::Address::Group(groups)) => groups.is_empty(),
        parser::HeaderValue::Empty => true,
        _ => false,
    }
}

pub(crate) fn push_builder_address<'a>(
    all_emails: &mut HashSet<Cow<'a, str>>,
    all_addrs: &mut Vec<builder::Address<'a>>,
    header: &'a parser::HeaderValue,
) {
    match header {
        parser::HeaderValue::Address(parser::Address::List(addrs)) => {
            for addr in addrs {
                if let Some(email) = addr.address.as_ref() {
                    if let Some(addr) = &addr.address {
                        if NO_REPLY.is_match(addr) {
                            continue;
                        }
                    }

                    if all_emails.insert(email.clone()) {
                        all_addrs.push(builder::Address::new_address(
                            addr.name.clone(),
                            email.clone(),
                        ))
                    }
                }
            }
        }
        parser::HeaderValue::Address(parser::Address::Group(groups)) => {
            for group in groups {
                if let Some(group_name) = group.name.as_ref() {
                    if all_emails.insert(group_name.clone()) {
                        let name = Some(group_name.clone());
                        let addrs = group
                            .addresses
                            .iter()
                            .filter_map(|addr| {
                                addr.address.as_ref().map(|email| {
                                    let name = addr.name.clone();
                                    let email = email.as_ref();
                                    builder::Address::new_address(name, email)
                                })
                            })
                            .collect();

                        all_addrs.push(builder::Address::new_group(name, addrs))
                    }
                }
            }
        }
        _ => (),
    }
}
