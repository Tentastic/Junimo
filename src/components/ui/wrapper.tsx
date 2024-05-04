import {clsx} from "clsx";
import {ModInfos} from "@models/mods.ts";


export default function Wrapper({mod, selected}: {mod: ModInfos, selected: boolean | undefined}) {
    return (
        <div
             className={clsx(
                 "w-full transform duration-150 cursor-pointer bg-muted hover:bg-muted-dark rounded-lg p-1 px-2 flex justify-between",
                 selected ? "text-lime-300 ring-1 ring-lime-300" : "",
                 mod.invisible && "hidden"
             )}>
            <p>{mod.name}</p>
            <p className="text-zinc-500">{mod.version}</p>
        </div>
    )
}