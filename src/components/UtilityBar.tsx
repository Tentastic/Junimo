import Logo from "@assets/Logo.webp";
import {Globe, Pause, Play, CopyPlus} from "lucide-react";
import {invoke} from "@tauri-apps/api/core";
import {Dispatch, SetStateAction} from "react";
import {Tooltip, TooltipContent, TooltipProvider, TooltipTrigger} from "@components/ui/tooltip.tsx";


export default function UtilityBar({playing, setPlaying}: {playing: boolean, setPlaying: Dispatch<SetStateAction<boolean>>}) {
    async function start() {
        setPlaying(true);
        await invoke('start_game');
    }

    async function stop() {
        await invoke('stop_game');
        setPlaying(false);
    }

    async function addMod() {
        await invoke("add_mod");
    }

    async function openBrowser() {
        await invoke('open_search_browser');
    }

    return (
        <div className="flex col-span-3 min-h-12 items-center justify-between">
            <div className="flex items-center gap-2 mb-2">
                <img src={Logo} alt="React Logo" className="w-10 h-10 select-none"/>
                <h1 className="text-4xl mt-2 text-[#8bc24a] font-bold select-none">Junimo</h1>
            </div>
            <div className="flex gap-2">
                <div>
                    <TooltipProvider>
                        <Tooltip>
                            <TooltipTrigger onClick={openBrowser} className="p-2 rounded-lg transition duration-150 bg-muted hover:bg-muted-dark">
                                <Globe size={24}/>
                            </TooltipTrigger>
                            <TooltipContent>
                                <p>Open NexusMods</p>
                            </TooltipContent>
                        </Tooltip>
                    </TooltipProvider>
                </div>
                <div>
                    <TooltipProvider>
                        <Tooltip>
                            <TooltipTrigger onClick={addMod} className="p-2 rounded-lg transition duration-150 bg-muted hover:bg-muted-dark">
                                <CopyPlus size={24}/>
                            </TooltipTrigger>
                            <TooltipContent>
                                <p>Add Mod</p>
                            </TooltipContent>
                        </Tooltip>
                    </TooltipProvider>
                </div>
                <div className="mr-16">
                    {playing ? (
                            <TooltipProvider>
                                <Tooltip>
                                    <TooltipTrigger onClick={stop} className="p-2 rounded-lg transition duration-150 bg-muted hover:bg-muted-dark">
                                        <Pause size={24}/>
                                    </TooltipTrigger>
                                    <TooltipContent>
                                        <p>Stop Game</p>
                                    </TooltipContent>
                                </Tooltip>
                            </TooltipProvider>
                        ) :
                        (
                            <TooltipProvider>
                                <Tooltip>
                                    <TooltipTrigger onClick={start} className="p-2 rounded-lg transition duration-150 bg-muted hover:bg-muted-dark">
                                    </TooltipTrigger>
                                    <TooltipContent>
                                        <p>Start Game</p>
                                    </TooltipContent>
                                </Tooltip>
                            </TooltipProvider>
                        )
                    }

                </div>
            </div>
        </div>
    )
}