mod header;

use yew::prelude::*;

#[function_component(Home)]
pub fn home() -> Html {
    html! {
        <>
            <header::Header />
            <style>
                { home_style() }
            </style>
            <main>
                <section>
                    <h1>{ "Welcome to H2 Technologies Peering Portal" }</h1>
                    <p>{ "This portal allows you to manage and monitor your peering sessions with H2 Technologies." }</p>
                </section>
                <section>
                    <h2>{ "Getting Started" }</h2>
                    <p>{ "To get started, navigate to the Peering Setup page to configure your peering sessions." }</p>
                </section>
                <section>
                    <h2>{ "Peering Sessions" }</h2>
                    <p>{ "View and manage your existing peering sessions on the Peering Sessions page." }</p>
                </section>
            </main>
        </>
    }
}

fn home_style() -> String {
    r#"
    main {
        max-width: 900px;
        margin: 2rem auto;
        padding: 0 2rem;
    }
    section {
        margin-bottom: 2rem;
    }
    h1 {
        font-size: 2.5rem;
        margin-bottom: 1rem;
    }
    h2 {
        font-size: 1.75rem;
        margin-bottom: 0.75rem;
    }
    p {
        font-size: 1.125rem;
        line-height: 1.6;
    }
    "#
    .to_string()
}