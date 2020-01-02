import "allocator/arena";

export { memory };

declare namespace typeConversion {
    function bytesToString(bytes: Uint8Array): string
}

declare namespace http {
    function get(url: String): Uint8Array
}

export function httpGetString(url: string): string {
    return typeConversion.bytesToString(http.get(url))
}

export function httpGet(url: string): Uint8Array {
    return http.get(url)
}
