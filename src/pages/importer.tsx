import {useEffect, useState} from "react";
import {invoke} from "@tauri-apps/api/core";
import {Label} from "@components/ui/label.tsx";
import {Input} from "@components/ui/input.tsx";
import {Folder} from "lucide-react";
import {clsx} from "clsx";
import JunimoDance from "../assets/JunimoDance.gif";
import {Checkbox} from "@components/ui/checkbox.tsx";

export default function Importer() {
    const [importPath, setImportPath] = useState("");
    const [valid, setValid] = useState(false);
    const [importing, setImporting] = useState(false);
    const [importAll, setImportAll] = useState(false);

    async function fetchPath() {
        const path = await invoke<string>('select_import_dir');
        setImportPath(path);
        if (path !== "") {
            setValid(true);
        }
        else {
            setValid(false);
        }
    }

    async function importProfile() {
        if (importPath === "") {
            return;
        }

        setImporting(true);
        const imported = await invoke<boolean>('import_profile', {path: importPath, all: importAll});
        if (imported) {
            await invoke<boolean>('close_import');
        }
    }


    return (
        <div className="w-full h-[100vh] p-6 flex flex-col justify-start items-start">
            <h1 className="text-3xl text-left font-bold mb-4 text-primary">Import</h1>
            <div className="flex flex-col mt-0 w-full">
                <Label htmlFor="path" className="ml-1 mb-1 text-lg text-muted-foreground">Path to .zip File</Label>
                <div className="flex gap-2">
                    <Input id="path" placeholder="Please enter your zip path..."
                           value={importPath} onChange={x => setImportPath(x.target.value)}/>
                    <button onClick={fetchPath} className="w-10 h-10 flex items-center justify-center transition duration-150 border rounded-lg
                        bg-muted hover:bg-muted-dark">
                        <Folder size={20}/>
                    </button>
                </div>
            </div>
            <div className="flex gap-1 mt-6">
                <Checkbox id="terms1" checked={importAll} onCheckedChange={e => setImportAll(!importAll)} />
                <div className="grid gap-1.5 leading-none">
                    <label
                        htmlFor="terms1"
                        className="text-sm font-medium leading-none peer-disabled:cursor-not-allowed peer-disabled:opacity-70"
                    >
                        Import an "All profiles" zip file
                    </label>
                    <p className="text-sm text-muted-foreground">
                        Warning: This will overwrite all your current profiles
                    </p>
                </div>
            </div>
            <div className="mt-auto w-full flex justify-end items-end">
                <button onClick={importProfile}
                        className={clsx(
                            "p-2 px-6 transition duration-150 bg-green-500 hover:bg-green-600 rounded-lg text-white relative",
                            !valid || importing ? "opacity-50 cursor-not-allowed" : ""
                        )}>
                    {importing && <img src={JunimoDance} alt="Junimo Dance" className="w-10 h-10 inline-block mr-2 absolute left-1/2 -translate-x-1/2 -top-7" /> }
                    Import
                </button>
            </div>
        </div>
    )
}