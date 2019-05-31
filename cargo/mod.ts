
const { dlopen, env } = Deno;

export interface CargoArtifact {
    output_path: string;
    is_lib: boolean;
    is_dylib: boolean;
    is_cdylib: boolean;
}

export enum CargoBuildVerbose {
    Standard = 0,
    Verbose = 1,
    VeryVerbose = 2,
}

export interface CargoBuildAllOptions {
    manifestPath: string;
    onlyLib: boolean;
    verbose: CargoBuildVerbose;
}

export const defaultCargoBuildOptions = {
    onlyLib: true,
    verbose: CargoBuildVerbose.Standard,
};

type CargoBuildReqKeys = Exclude<keyof CargoBuildAllOptions, keyof typeof defaultCargoBuildOptions>;

// TODO(afinch7) replace this with a Omit when we upgrade to typescript 3.5
// Fancy type expression for any key not in defaultCargoBuildOptions is required
export type CargoBuildOptions = Pick<CargoBuildAllOptions, CargoBuildReqKeys> & Partial<CargoBuildAllOptions>;

export interface CargoBuildRes {
    output_root: string;
    artifacts: Array<CargoArtifact>;
}

// TODO(afinch7) make DL_PATH_CARGO_LOAD optional once we can load libs via url
// and add default url to use here.
const dlib = dlopen(env().DL_PATH_CARGO_LOAD);
const cargoBuildOp = dlib.loadFn("build");

const textEncoder = new TextEncoder();
const textDecoder = new TextDecoder();

export function build(opts: CargoBuildOptions): CargoBuildRes {
    return JSON.parse(
        textDecoder.decode(
            cargoBuildOp.dispatchSync(
                textEncoder.encode(
                    JSON.stringify(
                        {
                            ...defaultCargoBuildOptions,
                            ...opts,
                        },
                    ),
                ),
            ),
        ),
    );
}