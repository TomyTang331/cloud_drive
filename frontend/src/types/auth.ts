export interface RegisterData {
    username: string;
    email: string;
    password: string;
}

export interface LoginData {
    username: string;
    password: string;
}

export interface AuthResponse {
    token: string;
    user_id: number;
    username: string;
    role: string;
}

export interface UserProfile {
    id: number;
    username: string;
    email: string;
    created_at: string;
}
