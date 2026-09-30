// Types partagés avec l'API Rust (voir backend/src/modules/*/model.rs).

export type Role = 'admin' | 'user' | 'subscriber';
export type ArticleStatus = 'draft' | 'validated' | 'published';

export interface User {
  id: string;
  email: string;
  name: string | null;
  role: Role;
  email_verified: boolean;
  is_active: boolean;
  has_password: boolean;
  last_login_at: string | null;
  created_at: string;
}

export interface ArticleSummary {
  slug: string;
  title: string;
  excerpt: string;
  cover_image: string | null;
  tags: string[];
  reading_minutes: number;
  published_at: string | null;
}

export interface PublicArticle extends ArticleSummary {
  content_html: string;
  updated_at: string;
  seo_title: string | null;
  seo_description: string | null;
}

export interface ArticlePage {
  items: ArticleSummary[];
  page: number;
  per_page: number;
  total: number;
  total_pages: number;
}

export interface TagCount {
  tag: string;
  count: number;
}

export interface Article {
  id: string;
  slug: string;
  title: string;
  excerpt: string;
  content_md: string;
  content_html: string;
  cover_image: string | null;
  tags: string[];
  reading_minutes: number;
  status: ArticleStatus;
  scheduled_at: string | null;
  published_at: string | null;
  newsletter_sent_at: string | null;
  seo_title: string | null;
  seo_description: string | null;
  created_at: string;
  updated_at: string;
}

export interface ArticleInput {
  title: string;
  slug: string | null;
  excerpt: string;
  content_md: string;
  cover_image: string | null;
  tags: string[];
  seo_title: string | null;
  seo_description: string | null;
}

export interface AdminStats {
  drafts: number;
  validated: number;
  published: number;
  users: number;
  subscribers: number;
}

export interface MessageResponse {
  message: string;
}

// ---------- Profil (format JSON Resume) ----------

export interface Profile {
  basics: {
    name: string;
    label: string;
    email: string;
    url: string;
    summary: string;
    location: { countryCode: string; address: string };
    profiles: { network: string; username: string; url: string }[];
  };
  work: WorkItem[];
  volunteer: (WorkItem & { organization: string })[];
  education: { institution: string; area: string; studyType: string; endDate: string }[];
  skills: { name: string; level: string; keywords: string[] }[];
  projects: ProjectItem[];
  interests: { name: string; keywords: string[] }[];
}

export interface WorkItem {
  name: string;
  company?: string;
  position: string;
  startDate: string;
  endDate: string;
  summary: string;
  highlights?: string[];
  location?: string;
}

export interface ProjectItem {
  name: string;
  description: string;
  startDate: string;
  url: string;
  keywords: string[];
  article?: string;
}
