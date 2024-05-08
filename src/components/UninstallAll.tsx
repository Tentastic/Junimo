import {Dispatch, SetStateAction, useState} from "react";
import {invoke} from "@tauri-apps/api/tauri";
import {
    Dialog, DialogClose,
    DialogContent,
    DialogDescription, DialogFooter,
    DialogHeader,
    DialogTitle,
    DialogTrigger
} from "@components/ui/dialog.tsx";
import {ModInfos} from "@models/mods.ts";


export default function UninstallAll({setKey, mods, numbers}: {setKey: Dispatch<SetStateAction<number>>, mods: ModInfos[], numbers: number[]}) {
    async function uninstallMods() {
        console.log(mods)
        console.log(numbers)
        let newMods: string[] = [];
        for (let i = 0; i < numbers.length; i++) {
            newMods.push(mods[numbers[i]].name);
            console.log(mods[numbers[i]].name);
        }

        await invoke("uninstall_mods", {mods: newMods});
        setKey(prevKey => prevKey + 1);
    }

    return (
        <>
            {numbers.length > 0 ? (
                <Dialog>
                    <DialogTrigger className="w-full">
                        <button className="w-full relative flex cursor-default select-none items-center rounded-sm px-2 py-1.5 text-sm
                                outline-none hover:bg-accent hover:text-accent-foreground data-[disabled]:pointer-events-none
                                data-[disabled]:opacity-50">Uninstall all
                        </button>
                    </DialogTrigger>
                    <DialogContent>
                        <DialogHeader>
                            <DialogTitle className="mb-4">Are you absolutely sure?</DialogTitle>
                            <DialogDescription>
                                This action cannot be undone. This will permanently delete multiple mods and all it's associated data.
                            </DialogDescription>
                        </DialogHeader>
                        <DialogFooter className="sm:justify-end">
                            <DialogClose asChild>
                                <button onClick={uninstallMods}
                                        className="transition duration-300 text-foreground bg-destructive hover:brightness-75 p-2 px-4 rounded">
                                    Delete
                                </button>
                            </DialogClose>
                        </DialogFooter>
                    </DialogContent>
                </Dialog>
            ) : (
                <button className="w-full relative flex cursor-default select-none items-center rounded-sm px-2 py-1.5 text-sm
                                outline-none hover:bg-accent hover:text-accent-foreground data-[disabled]:pointer-events-none
                                data-[disabled]:opacity-50 brightness-50">Uninstall all
                </button>
            )}
        </>
    )
}