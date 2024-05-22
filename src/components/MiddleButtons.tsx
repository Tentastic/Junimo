import {clsx} from "clsx";
import {ArrowLeft, ArrowRight} from "lucide-react";
import DancingJunimo from "@assets/JunimoDance.gif";
import {useModsState} from "@components/ModsProvider.tsx";


export default function MiddleButtons({playing}: {playing: boolean}) {
    const { addMods, removeMods, selectedAdd, selectedRemove } = useModsState();

    return (
        <div className="flex flex-col gap-2 h-full px-4 relative">
            <div className="flex flex-col gap-4 h-full items-center justify-center">
                <button onClick={addMods} className={clsx(
                    "h-12 w-12 rounded-full transition duration-150 bg-muted hover:bg-muted-dark flex items-center justify-center",
                    selectedAdd[0].length === 0 && "opacity-50 pointer-events-none"
                )}>
                    <ArrowLeft/>
                </button>
                <button onClick={removeMods} className={clsx(
                    "h-12 w-12 rounded-full transition duration-150 bg-muted hover:bg-muted-dark flex items-center justify-center",
                    selectedRemove[0].length === 0 && "opacity-50 pointer-events-none"
                )}>
                    <ArrowRight/>
                </button>
            </div>
            {playing && (
                <div className="h-12 w-full absolute -bottom-4">
                    <img src={DancingJunimo} alt="Dancing Junimo"/>
                </div>
            )}
        </div>
    )
}