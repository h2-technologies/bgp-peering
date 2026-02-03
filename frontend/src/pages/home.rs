use yew::prelude::*;

#[function_component(Home)]
pub fn home() -> Html {
    html! {
        <div class="home-page">
            <div class="hero">
                <h2>{ "Welcome to the BGP Peering Portal" }</h2>
                <p class="subtitle">
                    { "Request peering with networks where we have common presence" }
                </p>
            </div>
            
            <div class="features">
                <div class="feature-card">
                    <h3>{ "🔐 Secure Authentication" }</h3>
                    <p>{ "Login using your PeeringDB credentials" }</p>
                </div>
                
                <div class="feature-card">
                    <h3>{ "🌍 Location Matching" }</h3>
                    <p>{ "Find common facilities and locations" }</p>
                </div>
                
                <div class="feature-card">
                    <h3>{ "🤝 Easy Peering" }</h3>
                    <p>{ "Request peering where we both have presence" }</p>
                </div>
            </div>
            
            <div class="getting-started">
                <h3>{ "Getting Started" }</h3>
                <ol>
                    <li>{ "Login with your PeeringDB credentials" }</li>
                    <li>{ "Enter your ASN and our ASN to check for common locations" }</li>
                    <li>{ "View the results and submit a peering request" }</li>
                </ol>
            </div>
        </div>
    }
}
