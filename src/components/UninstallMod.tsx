import {invoke} from "@tauri-apps/api/tauri";
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


export default function UninstallMod({name, setKey}: {name: string, setKey: Dispatch<SetStateAction<number>>}) {
    async function uninstallMods() {
        await invoke<ModInfos[]>('uninstall_mod', {name: name});
        setKey(prevKey => prevKey + 1);
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