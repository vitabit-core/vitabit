AINDA NAO ESTA PRONTO !!!!!!!!!!!! <<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<


VitaBit
VitaBit é uma criptomoeda desenvolvida em Rust, com foco em segurança, transparência e escalabilidade. O projeto tem como objetivo ser uma moeda real, funcional no cotidiano, tanto em meios digitais quanto físicos, oferecendo uma alternativa moderna e descentralizada ao dinheiro tradicional.

✨ Visão
VitaBit não é apenas um experimento técnico — é uma iniciativa séria para criar uma criptomoeda utilizável, confiável e eficiente. Projetada desde o início para suportar o crescimento e adoção no mundo real, sua arquitetura modular e segura permite evoluir com o tempo.

💀 Regra dos 100 anos para carteiras mortas – explicação
Definição:
A regra de 100 anos estabelece que, se uma carteira de criptomoedas não realiza nenhuma movimentação por um período de 100 anos, seus fundos poderão ser considerados abandonados ou mortos, e poderão ser redistribuídos, reutilizados ou queimados, dependendo das regras do protocolo.

🔧 Tecnologias
Rust – Linguagem segura e de alta performance para sistemas críticos.

Criptografia ECC (secp256k1) – Padrão utilizado pelo Bitcoin para garantir segurança nas transações.

Estrutura modular – Códigos organizados por responsabilidade: carteira, blocos, transações, mineração, etc.

📦 Módulos principais
wallet – Criação e gerenciamento de carteiras criptográficas.

transaction – Estrutura e lógica das transações entre usuários.

block – Definição de blocos e seu encadeamento com hashes.

blockchain – Controle da cadeia de blocos, validação e adição de novos blocos.

miner – Mecanismo de prova de trabalho (PoW) e cálculo de hashes.

main.rs – Exemplo de execução da blockchain localmente.


🚀 Como rodar
Clone o repositório:
git clone https://github.com/seu-usuario/vitabit.git
cd vitabit


Compile o projeto:
cargo build


Execute:
cargo run


Pré-requisitos: Você precisa ter o Rust instalado. Se não tiver, instale com:
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

🛠️ Próximos passos
🧠 Lógica de validação de transações.

⛏️ Ajuste dinâmico de dificuldade de mineração.

🌐 Integração com rede P2P.

📱 Interface gráfica e aplicativo mobile.

📃 Auditorias externas e testes de segurança.

📜 Licença
Distribuído sob a licença MIT. Veja o arquivo LICENSE para mais informações.
