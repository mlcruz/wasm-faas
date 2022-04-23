#include "libwasmfaas.h"
#include "iostream"

int main()
{
    uint64_t runtime_id = initialize_runtime();
    std::cout << runtime_id << '\n';
}