import Splash from '../assets/Splash.png';

export default function Splashscreen() {
    return (
        <div className="w-full h-full overflow-hidden overflow-y-hidden">
            <img src={Splash} alt="Splashscreen" className="w-full h-full" />
        </div>
    )
}