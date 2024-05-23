import Splash from '../assets/Splash.png';
import {check} from "@tauri-apps/plugin-updater";
import {relaunch} from "@tauri-apps/plugin-process";
import {useEffect} from "react";

export default function Splashscreen() {
    return (
        <div className="w-full h-full overflow-hidden overflow-y-hidden">
            <img src={Splash} alt="Splashscreen" className="w-full h-full" />
        </div>
    )
}