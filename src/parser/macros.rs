#[allow(unused_macros)]
macro_rules! parse_inner {
    ($event:expr, $reader:expr, $buffer:expr, $($rest:tt)*) => ({
        loop {
            use $crate::quick_xml::events::BytesEnd;
            use $crate::quick_xml::events::BytesStart;
            use $crate::quick_xml::events::Event;
            use $crate::quick_xml::Error as XmlError;

            $buffer.clear();
            match $reader.read_event_into($buffer) {
                Ok(Event::Start(ref x)) => {
                    parse_inner_impl!(x, x.name(), $($rest)*);
                    $reader.read_to_end_into(x.name(), &mut Vec::new())?;
                    unimplemented!(
                        "`{}` in `{}`",
                        std::string::String::from_utf8_lossy(x.local_name().as_ref()),
                        std::string::String::from_utf8_lossy($event.local_name().as_ref())
                    );
                }
                Err(e) => {
                    return Err(Error::from(e));
                }
                Ok(Event::Eof) => {
                    let e = std::string::String::from_utf8_lossy($event.local_name().as_ref()).to_string();
                    return Err(Error::from(XmlError::UnexpectedEof(e)));
                }
                Ok(Event::End(ref e)) if e.name() == $event.name() => {
                    break;
                }
                Ok(Event::End(ref e)) => {
                    let expected = std::string::String::from_utf8_lossy($event.name().as_ref()).to_string();
                    let found = std::string::String::from_utf8_lossy(e.name().as_ref()).to_string();
                    let e = XmlError::EndEventMismatch { expected, found };
                    return Err(Error::from(e));
                }
                _ => continue,
            }
        }
    })
}

#[allow(unused_macros)]
macro_rules! parse_inner_impl {
    ( $x:ident, $name:expr ) => ();
    ( $x:ident, $name:expr, ) => ();
    ( $x:ident, $name:expr, $e:ident @ $l:expr => $r:expr ) => (
        if $name.as_ref() == $l.as_ref() {
            let $e = $x.clone().into_owned();
            $r;
            continue;
        }
    );
    ( $x:ident, $name:expr, $l:expr => $r:expr ) => (
        if $name.as_ref() == $l.as_ref() {
            $r;
            continue;
        }
    );
    ( $x:ident, $name:expr, $e:ident @ $l:expr => $r:expr, $($rest:tt)*) => (
        parse_inner_impl!( $x, $name, $e @ $l => $r );
        parse_inner_impl!( $x, $name, $($rest)* );
    );
    ( $x:ident, $name:expr, $l:expr => $r:expr, $($rest:tt)*) => (
        parse_inner_impl!( $x, $name, $l => $r );
        parse_inner_impl!( $x, $name, $($rest)* );
    )
}

#[allow(unused_macros)]
macro_rules! parse_comment {
    ( $event:ident, $reader:ident, $buffer:ident, $comment:ident ) => {
        parse_comment!{$event, $reader, $buffer, $comment, }
    };
    ( $event:ident, $reader:ident, $buffer:ident, $comment:ident, $($rest:tt)* ) => {
        parse_inner!{$event, $reader, $buffer,
            t @ b"text" => {
                $comment.text.push(parse_text!(&t, $reader, $buffer));
            },
            m @ b"molecule" => {
                $comment.molecule = Molecule::from_xml(&m, $reader, $buffer)
                    .map(Some)?;
            },
            $($rest)*
        }
    }
}

#[allow(unused_macros)]
macro_rules! parse_text {
    ( $event:expr, $reader:ident, $buffer:ident ) => {{
        let mut txt = crate::common::ShortString::default();

        loop {
            use $crate::quick_xml::events::BytesEnd;
            use $crate::quick_xml::events::BytesStart;
            use $crate::quick_xml::events::Event;
            use $crate::quick_xml::Error as XmlError;

            $buffer.clear();
            match $reader.read_event_into($buffer) {
                Ok(Event::Text(ref e)) => {
                    if txt.is_empty() {
                        txt = e.unescape()?.into();
                    } else {
                        txt.push_str(&e.unescape()?);
                    }
                }
                Ok(Event::Start(_)) => {
                    return Err(Error::from(XmlError::TextNotFound));
                }
                Err(e) => {
                    return Err(Error::from(e));
                }
                Ok(Event::Eof) => {
                    let e = std::string::String::from_utf8_lossy($event.local_name().as_ref())
                        .to_string();
                    return Err(Error::from(XmlError::UnexpectedEof(e)));
                }
                Ok(Event::End(ref e)) if e.name() == $event.name() => {
                    break;
                }
                Ok(Event::End(ref e)) => {
                    let expected =
                        std::string::String::from_utf8_lossy($event.name().as_ref()).to_string();
                    let found = std::string::String::from_utf8_lossy(e.name().as_ref()).to_string();
                    let e = XmlError::EndEventMismatch { expected, found };
                    return Err(Error::from(e));
                }
                _ => continue,
            }
        }

        txt
    }};
}
