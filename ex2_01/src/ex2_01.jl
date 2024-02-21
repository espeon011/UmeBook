using JuMP
import HiGHS
import CSV
import DataFrames

dir = mktempdir()

meatCsvFileName = joinpath(dir, "meat.csv")
open(meatCsvFileName, "w") do io
    write(
        io,
        """
        name,method,price
        もも肉,生肉,8
        もも肉,燻製/通常,14
        もも肉,燻製/超過,11
        バラ肉,生肉,4
        バラ肉,燻製/通常,12
        バラ肉,燻製/超過,7
        肩肉,生肉,4
        肩肉,燻製/通常,13
        肩肉,燻製/超過,9
        """,
    )
    return
end
meats = CSV.read(meatCsvFileName, DataFrames.DataFrame)
println(meats)

Base.prompt("Press any key")

model = Model(HiGHS.Optimizer)

@variable(model, quantity[1:size(meats, 1)] >= 0)
meats.quantity = quantity
println(meats)

Base.prompt("Press any key")

@objective(model, Max, sum(meats.price .* meats.quantity))

@constraint(model, sum(meats[meats.name .== "もも肉", :quantity]) <= 480)
@constraint(model, sum(meats[meats.name .== "バラ肉", :quantity]) <= 400)
@constraint(model, sum(meats[meats.name .== "肩肉", :quantity]) <= 230)

@constraint(model, sum(meats[meats.method .== "燻製/通常", :quantity]) <= 420)
@constraint(model, sum(meats[meats.method .== "燻製/超過", :quantity]) <= 250)

println(model)

Base.prompt("Press any key")

optimize!(model)

Base.prompt("Press any key")

meats.quantity .= map(q -> value(q), meats.quantity)
println(meats)
