/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! `NSDateFormatter`.
//!
//! Resources:
//! - Apple's [Introduction to Data Formatting Programming Guide For Cocoa](https://developer.apple.com/library/archive/documentation/Cocoa/Conceptual/DataFormatting/DataFormatting.html)
//! - [Unicode Technical Standard #35](https://unicode.org/reports/tr35/tr35-10.html#Date_Format_Patterns)

use crate::frameworks::core_foundation::time::{CFAbsoluteTimeGetGregorianDate, CFGregorianDate};
use crate::frameworks::foundation::{ns_string, NSTimeInterval};
use crate::objc::{
    autorelease, id, msg, msg_class, nil, objc_classes, release, retain, todo_objc_setter,
    ClassExports, HostObject, NSZonePtr,
};

struct NSDateFormatterHostObject {
    date_format: Option<id>,
    locale: id,
}
impl HostObject for NSDateFormatterHostObject {}

// Numeric Unicode date fields and English day periods, as used by Violet.
// Text month/day names, calendars and timezone formatting remain unsupported.
fn format_gregorian_date(pattern: &str, date: CFGregorianDate) -> Result<String, String> {
    let mut result = String::new();
    let mut chars = pattern.chars().peekable();
    let mut quoted = false;
    while let Some(ch) = chars.next() {
        if ch == '\'' {
            if chars.peek() == Some(&'\'') {
                chars.next();
                result.push('\'');
            } else {
                quoted = !quoted;
            }
            continue;
        }
        if quoted || !ch.is_ascii_alphabetic() {
            result.push(ch);
            continue;
        }
        let mut width = 1;
        while chars.peek() == Some(&ch) {
            chars.next();
            width += 1;
        }
        let value: i32 = match ch {
            // YYYY preserves the old formatter's calendar-year approximation.
            // A true week-based year requires calendar/week rule support.
            'y' | 'Y' => {
                if width == 2 {
                    date.year.rem_euclid(100)
                } else {
                    date.year
                }
            }
            'M' if width <= 2 => date.month.into(),
            'd' if width <= 2 => date.day.into(),
            'H' if width <= 2 => date.hours.into(),
            'h' if width <= 2 => {
                let hour = date.hours as i32 % 12;
                if hour == 0 {
                    12
                } else {
                    hour
                }
            }
            'm' if width <= 2 => date.minutes.into(),
            's' if width <= 2 => date.seconds.floor() as i32,
            'a' => {
                result.push_str(if date.hours < 12 { "AM" } else { "PM" });
                continue;
            }
            _ => {
                return Err(format!(
                    "unsupported date field {}",
                    ch.to_string().repeat(width)
                ))
            }
        };
        result.push_str(&format!("{value:0width$}"));
    }
    if quoted {
        return Err("unterminated date-format literal".to_string());
    }
    Ok(result)
}

pub const CLASSES: ClassExports = objc_classes! {

(env, this, _cmd);

@implementation NSDateFormatter: NSObject

+ (id)allocWithZone:(NSZonePtr)_zone {
    let host_object = Box::new(NSDateFormatterHostObject {
        date_format: None,
        locale: nil,
    });
    env.objc.alloc_object(this, host_object, &mut env.mem)
}

- (())setDateFormat:(id)format { // NSString *
    let date_format: id = msg![env; format copy];
    let previous = env.objc.borrow_mut::<NSDateFormatterHostObject>(this).date_format.replace(date_format);
    if let Some(previous) = previous {
        release(env, previous);
    }
}

- (())setLocale:(id)locale {
    retain(env, locale);
    let previous = std::mem::replace(&mut env.objc.borrow_mut::<NSDateFormatterHostObject>(this).locale, locale);
    release(env, previous);
}

- (id)locale {
    let locale = env.objc.borrow::<NSDateFormatterHostObject>(this).locale;
    if locale == nil {
        msg_class![env; NSLocale currentLocale]
    } else {
        locale
    }
}

- (())dealloc {
    let &NSDateFormatterHostObject { date_format, locale } = env.objc.borrow(this);
    if let Some(date_format) = date_format {
        release(env, date_format);
    }
    release(env, locale);
    env.objc.dealloc_object(this, &mut env.mem)
}

- (())setTimeZone:(id)time_zone {
    todo_objc_setter!(this, time_zone);
}

- (id)stringFromDate:(id)date {
    let &NSDateFormatterHostObject {
        date_format, ..
    } = env.objc.borrow(this);
    let pattern = ns_string::to_rust_string(env, date_format.unwrap()).into_owned();
    log!("NSDateFormatter format: {:?}", pattern);
    let ti: NSTimeInterval = msg![env; date timeIntervalSinceReferenceDate];
    let greg_date = CFAbsoluteTimeGetGregorianDate(env, ti, nil);
    let format = format_gregorian_date(&pattern, greg_date)
        .unwrap_or_else(|error| panic!("NSDateFormatter: {error}"));

    let res = ns_string::from_rust_string(env, format);
    autorelease(env, res)
}

@end

};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn numeric_date_patterns_and_literals() {
        let date = CFGregorianDate {
            year: 2026,
            month: 10,
            day: 18,
            hours: 0,
            minutes: 5,
            seconds: 9.75,
        };
        assert_eq!(
            format_gregorian_date("yyyy-MM-dd hh:mm:ss a", date).unwrap(),
            "2026-10-18 12:05:09 AM"
        );
        assert_eq!(
            format_gregorian_date("yy/M/d H:m:s", date).unwrap(),
            "26/10/18 0:5:9"
        );
        assert_eq!(
            format_gregorian_date("yyyy-MM-dd'T'HH:mm:ss", date).unwrap(),
            "2026-10-18T00:05:09"
        );
        assert_eq!(
            format_gregorian_date("'at' h 'o''clock'", date).unwrap(),
            "at 12 o'clock"
        );
        for (hours, expected) in [(1, "01 AM"), (12, "12 PM"), (13, "01 PM"), (23, "11 PM")] {
            let date = CFGregorianDate { hours, ..date };
            assert_eq!(format_gregorian_date("hh a", date).unwrap(), expected);
        }
        assert!(format_gregorian_date("MMMM", date).is_err());
        assert!(format_gregorian_date("'unclosed", date).is_err());
    }
}
