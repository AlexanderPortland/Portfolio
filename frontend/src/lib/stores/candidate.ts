import type { GradeBackend } from '$lib/components/grades/GradesTable.svelte';
import { writable } from 'svelte/store';

export interface SchoolJson {
	n: string;
	f: string[];
}
export interface School {
	name: string;
	field: string;
}
export interface CandidateData {
	candidate: {
		name: string;
		surname: string;
		birthSurname: string;
		birthplace: string;
		birthdate: string;
		address: string;
		letterAddress: string;
		telephone: string;
		citizenship: string;
		email: string;
		sex: string;
		personal_id_number: string;
		schoolName: string;
		healthInsurance: string;
		grades: Array<GradeBackend>;
		firstSchool: School;
		secondSchool: School;
		testLanguage: string;
	};
	parents: Array<{
		name: string;
		surname: string;
		telephone: string;
		email: string;
	}>;
}

export interface CandidatePreview {
	application_id?: number;
	candidate_id?: number;
	related_applications?: Array<number>;
	personal_id_number?: string;
	name?: string;
	surname?: string;
	email?: string;
	field_of_study?: string;
	created_at?: string;
}

export interface CandidateLogin {
	applicationId: number;
	password: string;
}

export interface CreateCandidate {
	applicationId: number;
	personalIdNumber: string;
}

export interface BaseCandidate {
	currentApplication: number;
	applications: Array<number>;
	personal_id_number: string;
	detailsFilled: boolean;
	encryptedBy?: number;
}

export interface CreateCandidateLogin {
	applicationId: number;
	personal_id_number: string;
	applications: [];
	fieldOfStudy: string;
	password: string;
}

export const baseCandidateData = writable<BaseCandidate>({
	currentApplication: 0,
	applications: [],
	personal_id_number: '',
	detailsFilled: false
});

export const candidateData = writable<CandidateData>({
	candidate: {
		name: '',
		surname: '',
		birthSurname: '',
		birthplace: '',
		birthdate: '',
		address: '',
		letterAddress: '',
		telephone: '',
		citizenship: '',
		email: '',
		sex: '',
		personal_id_number: '',
		schoolName: '',
		healthInsurance: '',
		grades: [],
		firstSchool: { name: '', field: '' },
		secondSchool: { name: '', field: '' },
		testLanguage: ''
	},
	parents: []
});
