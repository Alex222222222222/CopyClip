use yew::{function_component, html, Html, Properties};

#[derive(Debug, PartialEq, Properties)]
pub struct TimeDisplayProps {
    pub time: i64, // unix time epoch in seconds
}

pub fn time_display(props: &TimeDisplayProps) -> String {
    let time = chrono::NaiveDateTime::from_timestamp_opt(props.time, 0).unwrap();
    let time = chrono::DateTime::<chrono::Utc>::from_utc(time, chrono::Utc);
    let time = time.with_timezone(&chrono::Local);
    time.format("%Y-%m-%d %H:%M:%S").to_string()
}

#[function_component(TimeDisplay)]
pub fn time_display_html(props: &TimeDisplayProps) -> Html {
    let time = time_display(props);

    html! {
        <td class="border border-gray-200">{time}</td>
    }
}
