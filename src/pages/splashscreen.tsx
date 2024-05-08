

export default function Splashscreen() {
    return (
        <div className="splashscreen">
            <div className="splashscreen__content">
                <div className="splashscreen__logo">
                    <img src="assets/Logo.webp" alt="Logo" />
                </div>
                <div className="splashscreen__progress">
                    <div className="splashscreen__progress-bar"></div>
                </div>
                <div className="splashscreen__message">
                    <p>Loading...</p>
                </div>
            </div>
        </div>
    )
}