use chrono::NaiveDate;
use chrono::Datelike;
use crate::rofi::Rofi;

/// Give the number of day in a month
/// 
/// Returns a `u32` with the number of days of the month
/// 
/// Arguments:
/// 
/// * `month` - the month you want the number of days
/// * `year` - the year
fn day_in_month(month: u32, year: i32) -> u32{
    match month {
        1 => 31,
        2 => if (year%4 == 0 && year%100 != 0) || year%400 == 0 {29} else {28},
        3 => 31,
        4 => 30,
        5 => 31,
        6 => 30,
        7 => 31,
        8 => 31,
        9 => 30,
        10 => 31,
        11 => 30,
        12 => 31,
        _ => panic!("No month n°{}", month)
    }
}

/// Open a Rofi menu to select a date.
/// 
/// Returns `Some(NaiveDate)` if a date is selected, `None` if the user quitted
/// 
/// Arguments:
/// 
/// * `default_date` - The date that is shown by default
pub fn date_selector(default_date : NaiveDate) -> Option<NaiveDate> {
    let now = default_date;
    let year : i32;
    let month : u32;
    let day : u32;
    let year_list : Vec<String> = (now.year()..now.year()+10).map(|x| x.to_string()).collect();
    loop {
        let selected_year = Rofi::new().prompt("Year").run(year_list.clone()).unwrap();
        if year_list.contains(&selected_year) {
            year = year_list.iter().position(|r| r.eq(&selected_year)).unwrap() as i32 + now.year();
            break;
        } else if selected_year.eq("") {
            return None;
        }
    }
    let month_list : Vec<String> = vec!["Janvier","Février","Mars","Avril","Mai","Juin","Juillet","Août","Septembre","Octobre","Novembre","Décembre"]
        .iter()
        .map(|&s|String::from(s))
        .collect();
    let suggested_month = now.month();
    loop {
        let selected_month = Rofi::new().prompt("Month").selected(suggested_month-1).run(month_list.clone()).unwrap();
        if month_list.contains(&selected_month) {
            month = month_list.iter().position(|r| r.eq(&selected_month)).unwrap() as u32 + 1;
            break;
        } else if selected_month.eq("") {
            return None;
        }
    }
    let day_list : Vec<String> = (1..day_in_month(month, year)+1).map(|x| x.to_string()).collect();
    let suggested_day = if month == now.month() {now.day()-1} else {0};
    loop {
        let selected_day = Rofi::new().prompt("Day").selected(suggested_day).run(day_list.clone()).unwrap();
        if day_list.contains(&selected_day) {
            day = day_list.iter().position(|r| r.eq(&selected_day)).unwrap() as u32 + 1;
            break;
        } else if selected_day.eq("") {
            return None;
        }
    }
    let dt = NaiveDate::from_ymd(year, month, day);
    Some(dt)
}

#[cfg(test)]
mod day_in_month_tests {
    use super::*;


    /// Trying to get the number of days in a month that doesn't exist
    #[test]
    #[should_panic]
    fn unexisting_month() {
        day_in_month(0,2021);
    }

    /// Testing the number of days in different monthss
    #[test]
    fn days_of_months() {
        assert_eq!(day_in_month(1,2021), 31);
        assert_eq!(day_in_month(2,2021), 28);
        assert_eq!(day_in_month(3,2021), 31);
        assert_eq!(day_in_month(4,2021), 30);
        assert_eq!(day_in_month(5,2021), 31);
        assert_eq!(day_in_month(6,2021), 30);
        assert_eq!(day_in_month(7,2021), 31);
        assert_eq!(day_in_month(8,2021), 31);
        assert_eq!(day_in_month(9,2021), 30);
        assert_eq!(day_in_month(10,2021), 31);
        assert_eq!(day_in_month(11,2021), 30);
        assert_eq!(day_in_month(12,2021), 31);
    }

    /// Testing some bissextile or non bissextile years
    #[test]
    fn bissextile_years() {
        assert_eq!(day_in_month(2,1900), 28);
        assert_eq!(day_in_month(2,2000), 29);
        assert_eq!(day_in_month(2,2001), 28);
        assert_eq!(day_in_month(2,2004), 29);
        assert_eq!(day_in_month(2,2100), 28);
        assert_eq!(day_in_month(2,2400), 29);
    }
}