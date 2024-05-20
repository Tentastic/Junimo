import {invoke} from "@tauri-apps/api/core";
import {ModInfos} from "@models/mods.ts";
import {Dispatch, SetStateAction} from "react";
import {
    Dialog,
    DialogClose,
    DialogContent,
    DialogDescription,
    DialogFooter,
    DialogHeader,
    DialogTitle,
    DialogTrigger
} from "@components/ui/dialog.tsx";
import {useModsState} from "@components/ModsProvider.tsx";


export default function UninstallMod({name}: {name: string}) {
    const { reloadKey } = useModsState();

    async function uninstallMods() {
        await invoke<ModInfos[]>('uninstall_mod', {name: name});
        reloadKey[1](prevKey => prevKey + 1);
    }

    return (
        <Dialog>
            <DialogTrigger className="w-full">
                <button className="w-full relative flex cursor-default select-none items-center rounded-sm px-2 py-1.5 text-sm
                                outline-none hover:bg-accent hover:text-accent-foreground data-[disabled]:pointer-events-none
                                data-[disabled]:opacity-50">
                    Uninstall
                </button>
            </DialogTrigger>
            <DialogContent>
                <DialogHeader>
                    <DialogTitle className="mb-4">Are you absolutely sure?</DialogTitle>
                    <DialogDescription>
                        This action cannot be undone. This will permanently delete {name} and
                        all it's data.
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
    )
}