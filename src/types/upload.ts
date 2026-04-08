export interface UploadItem {
  id: number;
  file_name: string;
  status: "uploading" | "done" | "error";
  url?: string | null;
  message?: string | null;
}

export interface EnqueueUploadsResult {
  accepted_files: string[];
  rejected_directories: string[];
}
