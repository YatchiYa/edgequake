# Dataset pin

- HF dataset: `GraphRAG-Bench/GraphRAG-Bench`
- Snapshot revision: `dc3a111e77dbaf8bbaf51ef331f3cfc9b1b5c546`
- medical_questions.json: 2062 questions
- novel_questions.json: 2010 questions
- smoke fixture: `smoke_question_ids_v1.txt` (n=40, seed=42, 10 per question_type)
- medical-mid publish fixture: `medical_publish_question_ids_v1.txt` (n=200, 50/type; Acc SSOT)
- medical-full fixture: `medical_full_question_ids_v1.txt` (n=2062 = all medical; scale check, not Acc SSOT)
- core fixture: `core_question_ids_v1.txt` (n=2162 = all medical + 100 novel stratified)
