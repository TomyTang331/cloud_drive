export interface ApiResponse<T> {
    code: number;
    message: string;
    request_id: string;
    data: T;
}

export interface StorageInfo {
    used_bytes: number;
    total_bytes: number;
    usage_percentage: number;
}
