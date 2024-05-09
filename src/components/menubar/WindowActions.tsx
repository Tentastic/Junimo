import {X, Square, Copy, Minus} from "lucide-react";
import { Window } from '@tauri-apps/api/window';
import {useEffect, useState} from "react";

export default function WindowActions() {
    const [maxstate, setMaxstate] = useState(false);
    const appWindow = Window.getCurrent();


    async function minimize() {
        await appWindow.minimize();

    }

    async function maximize() {
        await appWindow.toggleMaximize();
        setMaxstate(await appWindow.isMaximized());
    }

    async function close() {
        await appWindow.close();
    }

    async function checkMaximized() {
        setMaxstate(await appWindow.isMaximized());

    }

    useEffect(() => {
        checkMaximized();
        appWindow.listen("tauri://resize", () => {
            checkMaximized();
        });
    }, []);

    return (
        <div className="w-32 h-full flex items-center">
            <button onClick={minimize} className="w-16 h-full flex items-center justify-center hover:bg-muted">
                <Minus size={16}/>
            </button>
            <button onClick={maximize} className="w-16 h-full flex items-center justify-center hover:bg-muted">
                {
                    maxstate ?
                        <Copy size={14} className="scale-x-[-1]"/>
                        :
                        <Square size={14}/>
                }
            </button>
            <button onClick={close} className="w-16 h-full flex items-center justify-center hover:bg-muted">
                <X size={18}/>
            </button>
        </div>
    )
}