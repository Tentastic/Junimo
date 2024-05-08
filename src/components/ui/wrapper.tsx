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
            <div className="flex gap-4">
                <p>{mod.name}</p>
                {mod.more_info !== undefined ? (
                    <div dangerouslySetInnerHTML={{__html: mod.more_info}} />
                ) : (
                    <></>
                )}
            </div>
            <p className="text-zinc-500">{mod.version}</p>
        </div>
    )
}