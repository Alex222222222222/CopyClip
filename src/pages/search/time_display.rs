use yew::{function_component, html, Html, Properties};

#[derive(Debug, PartialEq, Properties)]
pub struct TimeDisplayProps {
    pub time: i64, // unix time epoch in seconds
}

/// Convert to a formatted string
pub fn time_display(props: &TimeDisplayProps) -> String {
    let time = chrono::DateTime::from_timestamp(props.time, 0).unwrap();
    let time = time.with_timezone(&chrono::Local);
    time.format("%Y-%m-%d %H:%M:%S").to_string()
}

/// Used to display the clip added time in the search page
#[function_component(TimeDisplay)]
pub fn time_display_html(props: &TimeDisplayProps) -> Html {
    let time = time_display(props);

    html! {
        <td class="border border-gray-200 text-center">{time}</td>
    }
}
